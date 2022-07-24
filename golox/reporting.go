package main

import "fmt"

var hadError bool

func report(line uint, where string, message string) {
	fmt.Printf("[line %d] Error%s: %s\n", line, where, message)

	hadError = true
}
