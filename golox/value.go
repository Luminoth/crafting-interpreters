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
	ValueTypeCallable ValueType = 5
	ValueTypeInstance ValueType = 6
)

type Value struct {
	Type ValueType `json:"type"`

	AnyValue    interface{} `json:"any"`
	NumberValue float64     `json:"number"`
	StringValue string      `json:"string"`
	BoolValue   bool        `json:"bool"`

	CallableValue Callable `json:"callable"`

	InstanceValue *LoxInstance `json:"instance"`
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

	if v.Type == ValueTypeCallable {
		return v.CallableValue.String()
	}

	if v.Type == ValueTypeInstance {
		return v.InstanceValue.String()
	}

	fmt.Fprintf(os.Stderr, "Unsupported value type %v\n", v.Type)
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

func NewCallableValue(callable Callable) Value {
	return Value{
		Type:          ValueTypeCallable,
		CallableValue: callable,
	}
}

func NewInstanceValue(class *LoxClass) Value {
	return Value{
		Type:          ValueTypeInstance,
		InstanceValue: NewLoxInstance(class),
	}
}

type Values map[string]Value
