package main

import (
	"fmt"
	"os"
)

type ValueType int

const (
	ValueTypeNil    ValueType = 0
	ValueTypeAny    ValueType = 1
	ValueTypeNumber ValueType = 2
	ValueTypeString ValueType = 3
	ValueTypeBool   ValueType = 4
)

type Value struct {
	Type ValueType `json:"type"`

	AnyValue    interface{} `json:"any"`
	NumberValue float64     `json:"number"`
	StringValue string      `json:"string"`
	BoolValue   bool        `json:"bool"`
}

func (v Value) String() string {
	if v.Type == ValueTypeNil {
		return "nil"
	} else if v.Type == ValueTypeAny {
		return fmt.Sprintf("%v", v.AnyValue)
	} else if v.Type == ValueTypeNumber {
		return fmt.Sprintf("%g", v.NumberValue)
	} else if v.Type == ValueTypeString {
		return v.StringValue
	} else if v.Type == ValueTypeBool {
		return fmt.Sprintf("%t", v.BoolValue)
	}

	fmt.Printf("Unsupported value type %v", v.Type)
	os.Exit(1)
	return ""
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
