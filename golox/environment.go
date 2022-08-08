package main

import "fmt"

type Environment struct {
	Values map[string]Value `json:"values"`

	Enclosing *Environment `json:"enclosing,omitempty"`
}

func NewEnvironment() Environment {
	return Environment{
		Values: map[string]Value{},
	}
}

func NewEnvironmentScope(enclosing *Environment) *Environment {
	environment := NewEnvironment()
	environment.Enclosing = enclosing
	return &environment
}

func (e *Environment) Define(name string, value Value) {
	//fmt.Printf("Defining variable '%s' = %v\n", name, value)
	e.Values[name] = value
}

func (e *Environment) Assign(name *Token, value Value) (err error) {
	//fmt.Printf("Assigning variable '%s' = %v\n", name.Lexeme, value)
	if _, ok := e.Values[name.Lexeme]; ok {
		e.Values[name.Lexeme] = value
		return
	}

	if e.Enclosing != nil {
		return e.Enclosing.Assign(name, value)
	}

	err = &RuntimeError{
		Message: fmt.Sprintf("Undefined variable '%s'.", name.Lexeme),
		Token:   name,
	}
	return
}

func (e *Environment) Get(name *Token) (value Value, err error) {
	if val, ok := e.Values[name.Lexeme]; ok {
		value = val
		return
	}

	if e.Enclosing != nil {
		return e.Enclosing.Get(name)
	}

	err = &RuntimeError{
		Message: fmt.Sprintf("Undefined variable '%s'.", name.Lexeme),
		Token:   name,
	}
	return

}
