package main

import (
	"fmt"
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
	if expression.Value == nil {
		return "nil"
	}
	return fmt.Sprintf("%v", expression.Value)
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
