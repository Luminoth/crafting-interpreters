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

	run(string(bytes))

	if hadError {
		os.Exit(65)
	}

	if hadRuntimeError {
		os.Exit(70)
	}

	return
}

func runPrompt() (err error) {
	scanner := bufio.NewScanner(os.Stdin)
	for {
		fmt.Print("> ")
		if !scanner.Scan() {
			fmt.Println()
			return scanner.Err()
		}

		line := scanner.Text()
		run(line)

		hadError = false
		hadRuntimeError = false
	}

}

func run(source string) {
	scanner := NewScanner(source)
	scanner.ScanTokens()

	//fmt.Println(scanner.Tokens)

	parser := NewParser(scanner.Tokens)
	//expression := parser.ParseExpression()
	statements := parser.ParseProgram()

	if hadError {
		return
	}

	//fmt.Println((&ExpressionPrinter{}).Print(expression))

	interpreter := NewInterpreter()
	//interpreter.InterpretExpression(expression)
	interpreter.InterpretProgram(statements)
}
