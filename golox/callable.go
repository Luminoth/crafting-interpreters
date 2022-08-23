package main

import (
	"fmt"
	"time"
)

type Callable interface {
	Call(interpreter *Interpreter, arguments []Value) (*Value, error)

	String() string
}

type LoxFunction struct {
	declaration *FunctionStatement
	closure     *Environment
}

func NewLoxFunction(declaration *FunctionStatement, closure *Environment) *LoxFunction {
	return &LoxFunction{
		declaration: declaration,
		closure:     closure,
	}
}

func (f *LoxFunction) Name() string {
	return f.declaration.Name.Lexeme
}

func (f *LoxFunction) Arity() int {
	return len(f.declaration.Params)
}

func (f *LoxFunction) String() string {
	return fmt.Sprintf("<fn %s>", f.Name())
}

func (f *LoxFunction) Call(interpreter *Interpreter, arguments []Value) (value *Value, err error) {
	environment := NewEnvironmentScope(f.closure)
	for idx, param := range f.declaration.Params {
		environment.Define(param.Lexeme, arguments[idx])
	}

	value, err = interpreter.executeBlock(f.declaration.Body, environment)
	if err != nil {
		if returnErr, ok := err.(*ReturnError); ok {
			value = returnErr.Value
			err = nil
		}
	}
	return
}

/* Native functions */

type PrintFunction struct {
}

func (f *PrintFunction) Call(interpreter *Interpreter, arguments []Value) (*Value, error) {
	fmt.Println(arguments[0])

	// no return value here
	// because it looks weird to print things twice
	return nil, nil
}

func (f *PrintFunction) String() string {
	return "<native fn>"
}

type ClockFunction struct {
}

func (f *ClockFunction) Call(interpreter *Interpreter, arguments []Value) (*Value, error) {
	value := NewNumberValue(float64(time.Now().UnixMilli()) / 1000.0)
	return &value, nil
}

func (f *ClockFunction) String() string {
	return "<native fn>"
}
