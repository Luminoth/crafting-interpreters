package main

import "time"

type Callable interface {
	Call(interpreter *Interpreter, arguments []Value) (Value, error)
}

/* Native functions */

type ClockFunction struct {
}

func (f *ClockFunction) Call(interpreter *Interpreter, arguments []Value) (Value, error) {
	return NewNumberValue(float64(time.Now().UnixMilli()) / 1000.0), nil
}
