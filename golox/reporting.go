package main

import "fmt"

var hadError bool

func reportError(line int, message string) {
	report(line, "", message)
}

func report(line int, where string, message string) {
	fmt.Printf("[line %d] Error%s: %s", line, where, message)

	hadError = true
}
