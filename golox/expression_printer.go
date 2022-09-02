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
	return fmt.Sprintf("(= %s %s)", expression.Name.Lexeme, v), nil
}

func (p *ExpressionPrinter) VisitBinaryExpression(expression *BinaryExpression) (string, error) {
	return p.parenthesize(expression.Operator.Lexeme, expression.Left, expression.Right)
}

func (p *ExpressionPrinter) VisitTernaryExpression(expression *TernaryExpression) (string, error) {
	return p.parenthesize("ternary", expression.Condition, expression.True, expression.False)
}

func (p *ExpressionPrinter) VisitLogicalExpression(expression *LogicalExpression) (string, error) {
	return p.parenthesize(expression.Operator.Lexeme, expression.Left, expression.Right)
}

func (p *ExpressionPrinter) VisitUnaryExpression(expression *UnaryExpression) (string, error) {
	return p.parenthesize(expression.Operator.Lexeme, expression.Right)
}

func (p *ExpressionPrinter) VisitCallExpression(expression *CallExpression) (string, error) {
	v := []Expression{expression.Callee}
	v = append(v, expression.Arguments...)
	return p.parenthesize(expression.Paren.Lexeme, v...)
}

func (p *ExpressionPrinter) VisitGetExpression(expression *GetExpression) (string, error) {
	expr, err := p.parenthesize("get", expression.Object)
	if err != nil {
		return "", err
	}
	return fmt.Sprintf("%s.%s", expression.Name.Lexeme, expr), nil
}

func (p *ExpressionPrinter) VisitSetExpression(expression *SetExpression) (string, error) {
	expr, err := p.parenthesize("set", expression.Object)
	if err != nil {
		return "", err
	}

	v, err := expression.Value.AcceptString(p)
	if err != nil {
		return "", err
	}

	return fmt.Sprintf("(= %s.%s %s)", expression.Name.Lexeme, expr, v), nil
}

func (p *ExpressionPrinter) VisitSuperExpression(expression *SuperExpression) (string, error) {
	return fmt.Sprintf("%s.%s", expression.Keyword.Lexeme, expression.Method.Lexeme), nil
}

func (p *ExpressionPrinter) VisitThisExpression(expression *ThisExpression) (string, error) {
	return expression.Keyword.Lexeme, nil
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
