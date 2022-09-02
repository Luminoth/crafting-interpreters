package main

import "fmt"

type LoxFunction struct {
	Declaration   *FunctionStatement `json:"declaration"`
	Closure       *Environment       `json:"closure"`
	IsInitializer bool               `json:"is_initializer"`
}

func NewLoxFunction(declaration *FunctionStatement, closure *Environment, isInitializer bool) *LoxFunction {
	return &LoxFunction{
		Declaration:   declaration,
		Closure:       closure,
		IsInitializer: isInitializer,
	}
}

func (f *LoxFunction) Name() string {
	return f.Declaration.Name.Lexeme
}

func (f *LoxFunction) Arity() int {
	return len(f.Declaration.Params)
}

func (f *LoxFunction) Call(interpreter *Interpreter, arguments []*Value) (value *Value, err error) {
	environment := NewEnvironmentScope(f.Closure)
	for idx, param := range f.Declaration.Params {
		environment.Define(param.Lexeme, arguments[idx])
	}

	value, err = interpreter.executeBlock(f.Declaration.Body, environment)
	if err != nil {
		// returns are passed through errors
		// so check for that first
		if returnErr, ok := err.(*ReturnError); ok {
			err = nil

			if f.IsInitializer {
				// initialzers return 'this'
				// which should be in the enclosing environment
				value = f.Closure.GetAt(0, "this")
			} else {
				value = returnErr.Value
			}
			return
		}
		return
	}

	if f.IsInitializer {
		// initialzers return 'this'
		// which should be in the enclosing environment
		value = f.Closure.GetAt(0, "this")
		return
	}

	return
}

func (f *LoxFunction) String() string {
	return fmt.Sprintf("<fn %s>", f.Name())
}

func (f *LoxFunction) Bind(instance *LoxInstance) *LoxFunction {
	// wrap methods with a special environment containing 'this'
	environment := NewEnvironmentScope(f.Closure)
	value := NewInstanceValue(instance)
	environment.Define("this", &value)
	return NewLoxFunction(f.Declaration, environment, f.IsInitializer)
}
