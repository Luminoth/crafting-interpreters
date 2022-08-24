package main

type Callable interface {
	Arity() int
	Call(interpreter *Interpreter, arguments []Value) (*Value, error)

	String() string
}
