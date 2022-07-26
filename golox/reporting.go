package main

import "fmt"

var hadError bool
var hadRuntimeError bool

func report(line uint, where string, message string) {
	fmt.Printf("[line %d] Error%s: %s\n", line, where, message)

	hadError = true
}

func runtimeError(err error) {
	if runtimeError, ok := err.(*RuntimeError); ok {
		fmt.Printf("%s\n[line %d]\n", runtimeError.Error(), runtimeError.Token.Line)
	} else {
		fmt.Printf("%s", err.Error())

	}

	hadRuntimeError = true
}
