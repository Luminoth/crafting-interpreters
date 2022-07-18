package main

import (
	"fmt"
	"os"
	"strings"
)

type ExpressionPrinter struct {
}

func (p *ExpressionPrinter) Print(expression Expression) string {
	return fmt.Sprintf("%v", expression.Accept(p))
}

func (p *ExpressionPrinter) VisitBinaryExpression(expression *BinaryExpression) interface{} {
	return p.parenthesize(expression.Operator.Lexeme, expression.Left, expression.Right)
}

func (p *ExpressionPrinter) VisitGroupingExpression(expression *GroupingExpression) interface{} {
	return p.parenthesize("group", expression.Expression)
}

func (p *ExpressionPrinter) VisitLiteralExpression(expression *LiteralExpression) interface{} {
	if expression.Value.Type == LiteralTypeNone {
		return "nil"
	} else if expression.Value.Type == LiteralTypeNumber {
		return expression.Value.NumberValue
	} else if expression.Value.Type == LiteralTypeString {
		return expression.Value.StringValue
	}

	fmt.Printf("Unsupported literal type %v", expression.Value.Type)
	os.Exit(1)
	return nil
}

func (p *ExpressionPrinter) VisitUnaryExpression(expression *UnaryExpression) interface{} {
	return p.parenthesize(expression.Operator.Lexeme, expression.Right)
}

func (p *ExpressionPrinter) parenthesize(name string, expressions ...Expression) string {
	builder := strings.Builder{}

	builder.WriteRune('(')
	builder.WriteString(name)
	for _, expression := range expressions {
		builder.WriteRune(' ')
		builder.WriteString(fmt.Sprintf("%v", expression.Accept(p)))
	}
	builder.WriteRune(')')

	return builder.String()
}
