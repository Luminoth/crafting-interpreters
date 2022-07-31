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

func (e *Environment) Assign(name *Token, value Value) error {
	//fmt.Printf("Assigning variable '%s' = %v\n", name.Lexeme, value)
	if _, ok := e.Values[name.Lexeme]; ok {
		e.Values[name.Lexeme] = value
		return nil
	}

	return &RuntimeError{
		Message: fmt.Sprintf("Undefined variable '%s'.", name.Lexeme),
		Token:   name,
	}
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
