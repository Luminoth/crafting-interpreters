package main

import (
	"errors"
	"fmt"
)

var hadError bool
var hadRuntimeError bool

func report(line uint, where string, message string) {
	fmt.Printf("[line %d] Error%s: %s\n", line, where, message)

	hadError = true
}

func reportError(token *Token, message string) {
	if token.Type == EOF {
		report(token.Line, " at end", message)
	} else {
		report(token.Line, fmt.Sprintf(" at '%s'", token.Lexeme), message)
	}
}

func runtimeError(err error) {
	var runtimeError *RuntimeError
	if errors.As(err, &runtimeError) {
		if runtimeError.Token != nil {
			fmt.Printf("%s\n[line %d]\n", runtimeError.Error(), runtimeError.Token.Line)
		} else {
			fmt.Printf("%s\n", runtimeError.Error())
		}
	} else {
		fmt.Printf("Unexpected runtime error: %s\n", err.Error())
	}

	hadRuntimeError = true
}
