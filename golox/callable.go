package main

import (
	"fmt"
	"time"
)

type Callable interface {
	Call(interpreter *Interpreter, arguments []Value) (Value, error)
}

/* Native functions */

type PrintFunction struct {
}

func (f *PrintFunction) Call(interpreter *Interpreter, arguments []Value) (Value, error) {
	fmt.Println(arguments[0])

	// no return value here
	// because it looks weird to print things twice
	return NewNilValue(), nil
}

type ClockFunction struct {
}

func (f *ClockFunction) Call(interpreter *Interpreter, arguments []Value) (Value, error) {
	return NewNumberValue(float64(time.Now().UnixMilli()) / 1000.0), nil
}
