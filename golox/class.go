package main

import "fmt"

type Methods map[string]*LoxFunction

type LoxClass struct {
	Name    string  `json:"name"`
	Methods Methods `json:"methods"`
}

func NewLoxClass(name string, methods Methods) *LoxClass {
	return &LoxClass{
		Name:    name,
		Methods: methods,
	}
}

func (c LoxClass) String() string {
	return c.Name
}

func (c *LoxClass) Arity() int {
	return 0
}

func (c *LoxClass) FindMethod(name string) *LoxFunction {
	if method, ok := c.Methods[name]; ok {
		return method
	}
	return nil
}

func (c *LoxClass) Call(interpreter *Interpreter, arguments []Value) (*Value, error) {
	value := NewInstanceValue(c)
	return &value, nil
}

type LoxInstance struct {
	Class  *LoxClass `json:"class"`
	Fields Values    `json:"fields"`
}

func NewLoxInstance(class *LoxClass) *LoxInstance {
	return &LoxInstance{
		Class:  class,
		Fields: Values{},
	}
}

func (i LoxInstance) String() string {
	return fmt.Sprintf("%s instance", i.Class.Name)
}

func (i *LoxInstance) Get(name *Token) (value Value, err error) {
	if v, ok := i.Fields[name.Lexeme]; ok {
		value = v
		return
	}

	method := i.Class.FindMethod(name.Lexeme)
	if method != nil {
		value = NewCallableValue(method.Name(), method.Arity(), method)
		return
	}

	err = &RuntimeError{
		Message: fmt.Sprintf("Undefined property '%s'.", name.Lexeme),
		Token:   name,
	}
	return
}

func (i *LoxInstance) Set(name *Token, value Value) {
	i.Fields[name.Lexeme] = value
}
