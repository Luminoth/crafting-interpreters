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
	return
}

func runPrompt() (err error) {
	scanner := bufio.NewScanner(os.Stdin)
	for {
		fmt.Print("> ")
		if !scanner.Scan() {
			return scanner.Err()
		}
		run(scanner.Text())
	}

}

func run(source string) {
	scanner := NewScanner(source)
	tokens := scanner.ScanTokens()

	for _, token := range tokens {
		fmt.Println(token)
	}
}
