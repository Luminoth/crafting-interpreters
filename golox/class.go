package main

import "fmt"

type LoxClass struct {
	Name string `json:"name"`
}

func (c LoxClass) String() string {
	return c.Name
}

func (f *LoxClass) Arity() int {
	return 0
}

func (c *LoxClass) Call(interpreter *Interpreter, arguments []Value) (*Value, error) {
	value := NewInstanceValue(c)
	return &value, nil
}

type LoxInstance struct {
	Class  *LoxClass        `json:"class"`
	Fields map[string]Value `json:"fields"`
}

func (i LoxInstance) String() string {
	return fmt.Sprintf("%s instance", i.Class.Name)
}

func (i *LoxInstance) Get(name *Token) (value Value, err error) {
	if v, ok := i.Fields[name.Lexeme]; ok {
		value = v
		return
	}

	err = &RuntimeError{
		Message: fmt.Sprintf("Undefined property '%s'.", name.Lexeme),
		Token:   name,
	}
	return
}
