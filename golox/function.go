package main

import (
	"fmt"
)

type LoxFunction struct {
	Declaration *FunctionStatement `json:"declaration"`
	Closure     *Environment       `json:"closure"`
}

func NewLoxFunction(declaration *FunctionStatement, closure *Environment) *LoxFunction {
	return &LoxFunction{
		Declaration: declaration,
		Closure:     closure,
	}
}

func (f *LoxFunction) Name() string {
	return f.Declaration.Name.Lexeme
}

func (f *LoxFunction) Arity() int {
	return len(f.Declaration.Params)
}

func (f *LoxFunction) String() string {
	return fmt.Sprintf("<fn %s>", f.Name())
}

func (f *LoxFunction) Call(interpreter *Interpreter, arguments []Value) (value *Value, err error) {
	environment := NewEnvironmentScope(f.Closure)
	for idx, param := range f.Declaration.Params {
		environment.Define(param.Lexeme, arguments[idx])
	}

	value, err = interpreter.executeBlock(f.Declaration.Body, environment)
	if err != nil {
		if returnErr, ok := err.(*ReturnError); ok {
			value = returnErr.Value
			err = nil
		}
	}
	return
}

func (f *LoxFunction) Bind(instance *LoxInstance) *LoxFunction {
	// wrap methods with a special environment containing "this"
	environment := NewEnvironmentScope(f.Closure)
	environment.Define("this", NewInstanceValue(instance))
	return NewLoxFunction(f.Declaration, environment)
}
