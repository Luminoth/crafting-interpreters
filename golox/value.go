package main

import (
	"fmt"
	"os"
)

type ValueType int

// TODO: use stringer to generate this
func (t ValueType) String() string {
	return [...]string{
		"nil",
		"number",
		"string",
		"bool",
		"function",
		"class",
		"instance",
	}[t]
}

const (
	ValueTypeNil      ValueType = 0
	ValueTypeNumber   ValueType = 1
	ValueTypeString   ValueType = 2
	ValueTypeBool     ValueType = 3
	ValueTypeFunction ValueType = 4
	ValueTypeClass    ValueType = 5
	ValueTypeInstance ValueType = 6
)

type Value struct {
	Type ValueType `json:"type"`

	AnyValue    interface{} `json:"any"`
	NumberValue float64     `json:"number"`
	StringValue string      `json:"string"`
	BoolValue   bool        `json:"bool"`

	FunctionValue Callable `json:"function"`
	ClassValue    Callable `json:"class"`

	InstanceValue *LoxInstance `json:"instance"`
}

func (v Value) String() string {
	if v.Type == ValueTypeNil {
		return "nil"
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

	if v.Type == ValueTypeClass {
		return v.ClassValue.String()
	}

	if v.Type == ValueTypeInstance {
		return v.InstanceValue.String()
	}

	fmt.Fprintf(os.Stderr, "Unsupported value type %v\n", v.Type)
	os.Exit(1)
	return ""
}

func (v *Value) isTruthy() bool {
	switch v.Type {
	case ValueTypeNil:
		return false
	case ValueTypeBool:
		return v.BoolValue
	default:
		return true
	}
}

func (v *Value) Equals(other *Value) bool {
	switch v.Type {
	case ValueTypeNil:
		return other.Type == ValueTypeNil
	case ValueTypeNumber:
		if other.Type == ValueTypeNumber {
			return v.NumberValue == other.NumberValue
		} else {
			return false
		}
	case ValueTypeString:
		if other.Type == ValueTypeString {
			return v.StringValue == other.StringValue
		} else {
			return false
		}
	case ValueTypeBool:
		if other.Type == ValueTypeBool {
			return v.BoolValue == other.BoolValue
		} else {
			return false
		}
	case ValueTypeFunction:
		return v.FunctionValue == other.FunctionValue
	case ValueTypeClass:
		return v.ClassValue == other.ClassValue
	case ValueTypeInstance:
		return v.InstanceValue == other.InstanceValue
	default:
		return false
	}
}

func (v *Value) GetClassValue() *LoxClass {
	// NOTE: no error checking here
	// the caller is responsible for that
	return v.ClassValue.(*LoxClass)
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

func NewFunctionValue(function Callable) Value {
	return Value{
		Type:          ValueTypeFunction,
		FunctionValue: function,
	}
}

func NewClassValue(class Callable) Value {
	return Value{
		Type:       ValueTypeClass,
		ClassValue: class,
	}
}

func NewInstanceValue(instance *LoxInstance) Value {
	return Value{
		Type:          ValueTypeInstance,
		InstanceValue: instance,
	}
}
