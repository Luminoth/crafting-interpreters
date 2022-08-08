package main

import (
	"fmt"
	"os"
)

type ValueType int

const (
	ValueTypeNil      ValueType = 0
	ValueTypeAny      ValueType = 1
	ValueTypeNumber   ValueType = 2
	ValueTypeString   ValueType = 3
	ValueTypeBool     ValueType = 4
	ValueTypeFunction ValueType = 5
)

type FunctionType struct {
	Name     string   `json:"name"`
	Arity    int      `json:"arity"`
	Callable Callable `json:"callable"`
}

func (t FunctionType) String() string {
	return fmt.Sprintf("%s(%d)", t.Name, t.Arity)
}

type Value struct {
	Type ValueType `json:"type"`

	AnyValue    interface{} `json:"any"`
	NumberValue float64     `json:"number"`
	StringValue string      `json:"string"`
	BoolValue   bool        `json:"bool"`

	FunctionValue FunctionType `json:"function"`
}

func (v Value) String() string {
	if v.Type == ValueTypeNil {
		return "nil"
	}

	if v.Type == ValueTypeAny {
		return fmt.Sprintf("%v", v.AnyValue)
	}

	if v.Type == ValueTypeNumber {
		return fmt.Sprintf("%g", v.NumberValue)
	}

	if v.Type == ValueTypeString {
		return v.StringValue
	}

	if v.Type == ValueTypeBool {
		return fmt.Sprintf("%t", v.BoolValue)
	}

	if v.Type == ValueTypeFunction {
		return v.FunctionValue.String()
	}

	fmt.Printf("Unsupported value type %v\n", v.Type)
	os.Exit(1)
	return ""
}

func NewValue(literal LiteralValue) (value Value, err error) {
	switch literal.Type {
	case LiteralTypeNil:
		value = NewNilValue()
	case LiteralTypeNumber:
		value = NewNumberValue(literal.NumberValue)
	case LiteralTypeString:
		value = NewStringValue(literal.StringValue)
	case LiteralTypeBool:
		value = NewBoolValue(literal.BoolValue)
	default:
		err = fmt.Errorf("unsupported literal type %v", literal.Type)
	}

	return
}

func NewNilValue() Value {
	return Value{
		Type: ValueTypeNil,
	}
}

func NewAnyValue(value interface{}) Value {
	return Value{
		Type:     ValueTypeAny,
		AnyValue: value,
	}
}

func NewNumberValue(value float64) Value {
	return Value{
		Type:        ValueTypeNumber,
		NumberValue: value,
	}
}

func NewStringValue(value string) Value {
	return Value{
		Type:        ValueTypeString,
		StringValue: value,
	}
}

func NewBoolValue(value bool) Value {
	return Value{
		Type:      ValueTypeBool,
		BoolValue: value,
	}
}

func NewFunctionValue(name string, arity int, callable Callable) Value {
	return Value{
		Type: ValueTypeFunction,
		FunctionValue: FunctionType{
			Name:     name,
			Arity:    arity,
			Callable: callable,
		},
	}
}
