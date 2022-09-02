package main

type Callable interface {
	Name() string
	Arity() int

	// can return nil (void return)
	// TODO: we could get rid of a lot of pointers
	// if we had a Void type Value instead
	Call(interpreter *Interpreter, arguments []*Value) (*Value, error)

	String() string
}
