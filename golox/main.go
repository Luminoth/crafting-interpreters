package main

import (
	"bufio"
	"flag"
	"fmt"
	"io/ioutil"
	"os"
)

func main() {
	debug := flag.Bool("debug", false, "Enable debug output")
	// TODO: a 'strict' flag would be useful for passing the lox test harness

	flag.Parse()

	if len(flag.Args()) > 1 {
		fmt.Println("Usage: golox [script]")
		os.Exit(64)
	}

	var err error
	if len(flag.Args()) == 1 {
		err = runFile(flag.Args()[0], *debug)
	} else {
		err = runPrompt(*debug)
	}

	if err != nil {
		fmt.Println(err.Error())
		os.Exit(1)
	}
}

func runFile(filename string, debug bool) (err error) {
	bytes, err := ioutil.ReadFile(filename)
	if err != nil {
		return
	}

	interpreter := NewInterpreter(debug)

	run(&interpreter, string(bytes), false, debug)

	if hadError {
		os.Exit(65)
	}

	if hadRuntimeError {
		os.Exit(70)
	}

	return
}

func runPrompt(debug bool) (err error) {
	interpreter := NewInterpreter(debug)

	scanner := bufio.NewScanner(os.Stdin)
	for {
		fmt.Print("> ")
		if !scanner.Scan() {
			fmt.Println()
			return scanner.Err()
		}

		line := scanner.Text()
		run(&interpreter, line, true, debug)

		hadError = false
		hadRuntimeError = false
	}

}

func run(interpreter *Interpreter, source string, printExpressions bool, debug bool) {
	scanner := NewScanner(source, debug)
	scanner.ScanTokens()

	//fmt.Println(scanner.Tokens)

	parser := NewParser(scanner.Tokens, debug)
	statements := parser.Parse()

	if hadError {
		return
	}

	resolver := NewResolver(interpreter)
	resolver.Resolve(statements)

	if hadError {
		return
	}

	value := interpreter.Interpret(statements)
	if printExpressions && value != nil {
		fmt.Println(value.String())
	}
}
