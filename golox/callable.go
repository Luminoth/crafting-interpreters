package main

type Callable interface {
	Call(interpreter *Interpreter, arguments []Value) (*Value, error)

	String() string
}
