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
	Class *LoxClass `json:"class"`
}

func (i LoxInstance) String() string {
	return fmt.Sprintf("%s instance", i.Class.Name)
}
