package main

import (
	"fmt"
	"time"
)

type PrintFunction struct {
}

func (f *PrintFunction) Name() string {
	return "print"
}

func (f *PrintFunction) Arity() int {
	return 1
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

func (f *ClockFunction) Name() string {
	return "clock"
}

func (f *ClockFunction) Arity() int {
	return 0
}

func (f *ClockFunction) Call(interpreter *Interpreter, arguments []Value) (*Value, error) {
	value := NewNumberValue(float64(time.Now().UnixMilli()) / 1000.0)
	return &value, nil
}

func (f *ClockFunction) String() string {
	return "<native fn>"
}
