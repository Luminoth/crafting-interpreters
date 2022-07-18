package main

import (
	"testing"
)

func TestMExpressionPrinter(t *testing.T) {
	// -123 * (45.67)
	expectedResult := "(* (- 123) (group 45.67))"

	expression := BinaryExpression{
		Left: &UnaryExpression{
			Operator: &Token{
				Type:   Minus,
				Lexeme: "-",
			},
			Right: &LiteralExpression{
				Value: NewNumberLiteral(123),
			},
		},
		Operator: &Token{
			Type:   Star,
			Lexeme: "*",
		},
		Right: &GroupingExpression{
			Expression: &LiteralExpression{
				Value: NewNumberLiteral(45.67),
			},
		},
	}

	printer := ExpressionPrinter{}
	result := printer.Print(&expression)
	if result != expectedResult {
		t.Fatalf("Print failed - expected %s, got %s", expectedResult, result)
	}
}
