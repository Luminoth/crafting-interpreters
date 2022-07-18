package main

import (
	"fmt"
	"os"
	"strings"
)

type ExpressionPrinter struct {
}

func (p *ExpressionPrinter) Print(expression Expression) string {
	return expression.AcceptString(p)
}

func (p *ExpressionPrinter) VisitBinaryExpression(expression *BinaryExpression) string {
	return p.parenthesize(expression.Operator.Lexeme, expression.Left, expression.Right)
}

func (p *ExpressionPrinter) VisitGroupingExpression(expression *GroupingExpression) string {
	return p.parenthesize("group", expression.Expression)
}

func (p *ExpressionPrinter) VisitLiteralExpression(expression *LiteralExpression) string {
	if expression.Value.Type == LiteralTypeNone {
		return "nil"
	} else if expression.Value.Type == LiteralTypeNumber {
		return fmt.Sprintf("%g", expression.Value.NumberValue)
	} else if expression.Value.Type == LiteralTypeString {
		return expression.Value.StringValue
	}

	fmt.Printf("Unsupported literal type %v", expression.Value.Type)
	os.Exit(1)
	return ""
}

func (p *ExpressionPrinter) VisitUnaryExpression(expression *UnaryExpression) string {
	return p.parenthesize(expression.Operator.Lexeme, expression.Right)
}

func (p *ExpressionPrinter) parenthesize(name string, expressions ...Expression) string {
	builder := strings.Builder{}

	builder.WriteRune('(')
	builder.WriteString(name)
	for _, expression := range expressions {
		builder.WriteRune(' ')
		builder.WriteString(expression.AcceptString(p))
	}
	builder.WriteRune(')')

	return builder.String()
}
