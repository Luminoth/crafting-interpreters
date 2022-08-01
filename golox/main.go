package main

import (
	"bufio"
	"flag"
	"fmt"
	"io/ioutil"
	"os"
)

func main() {
	flag.Parse()

	if len(flag.Args()) > 1 {
		fmt.Println("Usage: golox [script]")
		os.Exit(64)
	}

	var err error
	if len(flag.Args()) == 1 {
		err = runFile(flag.Args()[0])
	} else {
		err = runPrompt()
	}

	if err != nil {
		fmt.Println(err.Error())
		os.Exit(1)
	}
}

func runFile(filename string) (err error) {
	bytes, err := ioutil.ReadFile(filename)
	if err != nil {
		return
	}

	interpreter := NewInterpreter()

	run(&interpreter, string(bytes), false)

	if hadError {
		os.Exit(65)
	}

	if hadRuntimeError {
		os.Exit(70)
	}

	return
}

func runPrompt() (err error) {
	interpreter := NewInterpreter()

	scanner := bufio.NewScanner(os.Stdin)
	for {
		fmt.Print("> ")
		if !scanner.Scan() {
			fmt.Println()
			return scanner.Err()
		}

		line := scanner.Text()
		run(&interpreter, line, true)

		hadError = false
		hadRuntimeError = false
	}

}

func run(interpreter *Interpreter, source string, printExpressions bool) {
	scanner := NewScanner(source)
	scanner.ScanTokens()

	//fmt.Println(scanner.Tokens)

	parser := NewParser(scanner.Tokens)
	statements := parser.Parse()

	if hadError {
		return
	}

	value := interpreter.Interpret(statements)
	if printExpressions && value != nil {
		fmt.Println(value.String())
	}
}
