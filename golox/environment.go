package main

import "fmt"

type Environment struct {
	Values map[string]Value `json:"values"`
}

func NewEnvironment() Environment {
	return Environment{
		Values: map[string]Value{},
	}
}

func (e *Environment) Define(name string, value Value) {
	//fmt.Printf("Defining variable '%s' = %v\n", name, value)
	e.Values[name] = value
}

func (e *Environment) Get(name *Token) (Value, error) {
	if val, ok := e.Values[name.Lexeme]; ok {
		return val, nil
	}

	return Value{}, &RuntimeError{
		Message: fmt.Sprintf("Undefined variable '%s'.", name.Lexeme),
		Token:   name,
	}
}
