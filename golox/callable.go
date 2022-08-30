package main

type Callable interface {
	Name() string
	Arity() int
	Call(interpreter *Interpreter, arguments []*Value) (*Value, error)

	String() string
}
