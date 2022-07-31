package main

import "testing"

func TestDefine(t *testing.T) {
	environment := NewEnvironment()

	token := &Token{Lexeme: "foo"}
	environment.Define(token.Lexeme, NewBoolValue(true))

	val, err := environment.Get(token)
	if err != nil {
		t.Fatalf("Define failed: %s", err)
	}

	if val.Type != ValueTypeBool || !val.BoolValue {
		t.Fatalf("Invalid defined value %s", val)
	}
}

func TestReefine(t *testing.T) {
	environment := NewEnvironment()

	token := &Token{Lexeme: "foo"}
	environment.Define(token.Lexeme, NewBoolValue(true))
	environment.Define(token.Lexeme, NewStringValue("bar"))

	val, err := environment.Get(token)
	if err != nil {
		t.Fatalf("Redefine failed: %s", err)
	}

	if val.Type != ValueTypeString || val.StringValue != "bar" {
		t.Fatalf("Invalid redefined value %s", val)
	}
}

func TestAssign(t *testing.T) {
	environment := NewEnvironment()

	token := &Token{Lexeme: "foo"}
	environment.Define(token.Lexeme, NewBoolValue(true))

	err := environment.Assign(token, NewStringValue("bar"))
	if err != nil {
		t.Fatalf("Assign failed: %s", err)
	}

	val, err := environment.Get(token)
	if err != nil {
		t.Fatalf("Assign failed: %s", err)
	}

	if val.Type != ValueTypeString || val.StringValue != "bar" {
		t.Fatalf("Invalid assigned value %s", val)
	}
}

func TestInvalidAssign(t *testing.T) {
	environment := NewEnvironment()

	token := &Token{Lexeme: "foo"}

	err := environment.Assign(token, NewStringValue("bar"))
	if err == nil {
		t.Fatalf("Assign did not fail")
	}

	_, err = environment.Get(token)
	if err == nil {
		t.Fatalf("Assign did not fail")
	}
}
