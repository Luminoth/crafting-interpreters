package main

import "fmt"

var hadError bool

func reportError(line uint, message string) {
	report(line, "", message)
}

func report(line uint, where string, message string) {
	fmt.Printf("[line %d] Error%s: %s\n", line, where, message)

	hadError = true
}
