package main

import (
	"fmt"
	"os"
)

type TokenType uint

// TODO: use stringer to generate this
func (t TokenType) String() string {
	return [...]string{
		"EOF",

		"LeftParen", "RightParen",
		"LeftBrace", "RightBrace",
		"Comma", "Dot",
		"Minus", "Plus",
		"Semicolon", "Slash", "Star",
		"Question", "Colon",

		"Bang", "BangEqual",
		"Equal", "EqualEqual",
		"Greater", "GreaterEqual",
		"Less", "LessEqual",

		"Identifier", "String", "Number",

		"And", "Or",
		"If", "Else",
		"Class", "Super", "This",
		"True", "False",
		"Fun",
		"For", "While",
		"Break", "Continue",
		"Nil",
		"Print",
		"Return",
		"Var",
	}[t]
}

const (
	EOF TokenType = 0

	// single character tokens
	LeftParen  TokenType = 1
	RightParen TokenType = 2
	LeftBrace  TokenType = 3
	RightBrace TokenType = 4
	Comma      TokenType = 5
	Dot        TokenType = 6
	Minus      TokenType = 7
	Plus       TokenType = 8
	Semicolon  TokenType = 9
	Slash      TokenType = 10
	Star       TokenType = 11
	Question   TokenType = 12
	Colon      TokenType = 13

	// one or two character tokens
	Bang         TokenType = 14
	BangEqual    TokenType = 15
	Equal        TokenType = 16
	EqualEqual   TokenType = 17
	Greater      TokenType = 18
	GreaterEqual TokenType = 19
	Less         TokenType = 20
	LessEqual    TokenType = 21

	// literals
	Identifier TokenType = 22
	String     TokenType = 23
	Number     TokenType = 24

	// keywords
	And      TokenType = 25
	Or       TokenType = 26
	If       TokenType = 27
	Else     TokenType = 28
	Class    TokenType = 29
	Super    TokenType = 30
	This     TokenType = 31
	True     TokenType = 32
	False    TokenType = 33
	Fun      TokenType = 34
	For      TokenType = 35
	While    TokenType = 36
	Break    TokenType = 37
	Continue TokenType = 38
	Nil      TokenType = 39
	Print    TokenType = 40
	Return   TokenType = 41
	Var      TokenType = 42
)

type LiteralType int

const (
	LiteralTypeNil    LiteralType = 0
	LiteralTypeNumber LiteralType = 1
	LiteralTypeString LiteralType = 2
	LiteralTypeBool   LiteralType = 3
)

type LiteralValue struct {
	Type LiteralType `json:"type"`

	NumberValue float64 `json:"number"`
	StringValue string  `json:"string"`
	BoolValue   bool    `json:"bool"`
}

func (v LiteralValue) String() string {
	if v.Type == LiteralTypeNil {
		return "nil"
	}

	if v.Type == LiteralTypeNumber {
		return fmt.Sprintf("%g", v.NumberValue)
	}

	if v.Type == LiteralTypeString {
		return v.StringValue
	}

	if v.Type == LiteralTypeBool {
		return fmt.Sprintf("%t", v.BoolValue)
	}

	fmt.Fprintf(os.Stderr, "Unsupported literal type %v\n", v.Type)
	os.Exit(1)
	return ""
}

func NewNilLiteral() LiteralValue {
	return LiteralValue{
		Type: LiteralTypeNil,
	}
}

func NewNumberLiteral(value float64) LiteralValue {
	return LiteralValue{
		Type:        LiteralTypeNumber,
		NumberValue: value,
	}
}

func NewStringLiteral(value string) LiteralValue {
	return LiteralValue{
		Type:        LiteralTypeString,
		StringValue: value,
	}
}

func NewBoolLiteral(value bool) LiteralValue {
	return LiteralValue{
		Type:      LiteralTypeBool,
		BoolValue: value,
	}
}

type Token struct {
	Type    TokenType    `json:"type"`
	Lexeme  string       `json:"lexeme,omitempty"`
	Literal LiteralValue `json:"literal,omitempty"`
	Line    uint         `json:"line"`
}

func (t Token) String() string {
	return fmt.Sprintf("[%d] %s '%s' '%s'", t.Line, t.Type, t.Lexeme, t.Literal)
}
