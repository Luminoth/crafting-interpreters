package main

import (
	"fmt"
	"strings"
)

type ExpressionPrinter struct {
}

func (p *ExpressionPrinter) Print(expression Expression) (string, error) {
	return expression.AcceptString(p)
}

func (p *ExpressionPrinter) VisitAssignExpression(expression *AssignExpression) (string, error) {
	v, err := expression.Value.AcceptString(p)
	if err != nil {
		return "", err
	}
	return fmt.Sprintf("(assign %s %s)", expression.Name.Lexeme, v), nil
}

func (p *ExpressionPrinter) VisitBinaryExpression(expression *BinaryExpression) (string, error) {
	return p.parenthesize(expression.Operator.Lexeme, expression.Left, expression.Right)
}

func (p *ExpressionPrinter) VisitTernaryExpression(expression *TernaryExpression) (string, error) {
	return p.parenthesize("ternary", expression.Condition, expression.True, expression.False)
}

func (p *ExpressionPrinter) VisitUnaryExpression(expression *UnaryExpression) (string, error) {
	return p.parenthesize(expression.Operator.Lexeme, expression.Right)
}

func (p *ExpressionPrinter) VisitGroupingExpression(expression *GroupingExpression) (string, error) {
	return p.parenthesize("group", expression.Expression)
}

func (p *ExpressionPrinter) VisitLiteralExpression(expression *LiteralExpression) (string, error) {
	return expression.Value.String(), nil
}

func (p *ExpressionPrinter) VisitVariableExpression(expression *VariableExpression) (string, error) {
	return expression.Name.Lexeme, nil
}

func (p *ExpressionPrinter) parenthesize(name string, expressions ...Expression) (string, error) {
	builder := strings.Builder{}

	builder.WriteRune('(')
	builder.WriteString(name)
	for _, expression := range expressions {
		builder.WriteRune(' ')

		v, err := expression.AcceptString(p)
		if err != nil {
			return "", err
		}
		builder.WriteString(v)
	}
	builder.WriteRune(')')

	return builder.String(), nil
}
