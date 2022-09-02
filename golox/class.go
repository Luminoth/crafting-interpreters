package main

import "fmt"

type Methods map[string]*LoxFunction

type LoxClass struct {
	ClassName  string    `json:"name"`
	Superclass *LoxClass `json:"superclass,omitempty"`
	Methods    Methods   `json:"methods"`
}

func NewLoxClass(name string, superclass *LoxClass, methods Methods) *LoxClass {
	return &LoxClass{
		ClassName:  name,
		Superclass: superclass,
		Methods:    methods,
	}
}

func (c *LoxClass) Name() string {
	return c.ClassName
}

func (c *LoxClass) Arity() int {
	initializer := c.FindMethod("init")
	if initializer == nil {
		return 0
	}
	return initializer.Arity()
}

func (c *LoxClass) Call(interpreter *Interpreter, arguments []*Value) (*Value, error) {
	instance := NewLoxInstance(c)

	// call initializer if there is one
	initializer := c.FindMethod("init")
	if initializer != nil {
		initializer.Bind(instance).Call(interpreter, arguments)
	}

	value := NewInstanceValue(instance)
	return &value, nil
}

func (c LoxClass) String() string {
	return c.Name()
}

func (c *LoxClass) FindMethod(name string) *LoxFunction {
	if method, ok := c.Methods[name]; ok {
		return method
	}

	// check superclass
	if c.Superclass != nil {
		return c.Superclass.FindMethod(name)
	}

	return nil
}

type Fields map[string]*Value

type LoxInstance struct {
	Class  *LoxClass `json:"class"`
	Fields Fields    `json:"fields"`
}

func NewLoxInstance(class *LoxClass) *LoxInstance {
	return &LoxInstance{
		Class:  class,
		Fields: Fields{},
	}
}

func (i LoxInstance) String() string {
	return fmt.Sprintf("%s instance", i.Class.Name())
}

func (i *LoxInstance) Get(name *Token) (value *Value, err error) {
	// check fields
	if v, ok := i.Fields[name.Lexeme]; ok {
		value = v
		return
	}

	// check methods
	method := i.Class.FindMethod(name.Lexeme)
	if method != nil {
		v := NewFunctionValue(method.Bind(i))
		value = &v
		return
	}

	err = &RuntimeError{
		Message: fmt.Sprintf("Undefined property '%s'.", name.Lexeme),
		Token:   name,
	}
	return
}

func (i *LoxInstance) Set(name *Token, value *Value) {
	i.Fields[name.Lexeme] = value
}
