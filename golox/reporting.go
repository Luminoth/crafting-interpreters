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
		if runtimeError.Token != nil {
			fmt.Printf("%s\n[line %d]\n", runtimeError.Error(), runtimeError.Token.Line)
		} else {
			fmt.Printf("%s\n", runtimeError.Error())
		}
	} else {
		fmt.Printf("%s\n", err.Error())

	}

	hadRuntimeError = true
}
