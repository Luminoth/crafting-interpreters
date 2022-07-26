package main

import (
	"testing"
)

func TestBinaryExpression(t *testing.T) {
	// -123 * (45.67)
	expectedResult := "(* (- 123) (group 45.67))"

	tokens := []*Token{
		{
			Type:   Minus,
			Lexeme: "-",
			Line:   1,
		},
		{
			Type:    Number,
			Lexeme:  "123",
			Literal: NewNumberLiteral(123),
			Line:    1,
		},
		{
			Type:   Star,
			Lexeme: "*",
			Line:   1,
		},
		{
			Type:   LeftParen,
			Lexeme: "(",
			Line:   1,
		},
		{
			Type:    Number,
			Lexeme:  "45.67",
			Literal: NewNumberLiteral(45.67),
			Line:    1,
		},
		{
			Type:   RightParen,
			Lexeme: ")",
			Line:   1,
		},
		{
			Type: EOF,
			Line: 1,
		},
	}

	parser := NewParser(tokens)
	expression := parser.Parse()

	result, err := (&ExpressionPrinter{}).Print(expression)
	if err != nil {
		t.Fatalf("Print failed: %s", err)
	}

	if result != expectedResult {
		t.Fatalf("Print failed - expected %s, got %s", expectedResult, result)
	}
}
