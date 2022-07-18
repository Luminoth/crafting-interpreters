package main

type Expression interface {
	AcceptString(visitor ExpressionVisitor) string
}

type BinaryExpression struct {
	Left     Expression
	Operator *Token
	Right    Expression
}

func (e *BinaryExpression) AcceptString(visitor ExpressionVisitor) string {
	return visitor.VisitBinaryExpressionString(e)
}

type GroupingExpression struct {
	Expression Expression
}

func (e *GroupingExpression) AcceptString(visitor ExpressionVisitor) string {
	return visitor.VisitGroupingExpressionString(e)
}

type LiteralExpression struct {
	Value LiteralValue
}

func (e *LiteralExpression) AcceptString(visitor ExpressionVisitor) string {
	return visitor.VisitLiteralExpressionString(e)
}

type UnaryExpression struct {
	Operator *Token
	Right    Expression
}

func (e *UnaryExpression) AcceptString(visitor ExpressionVisitor) string {
	return visitor.VisitUnaryExpressionString(e)
}

type ExpressionVisitor interface {
	VisitBinaryExpressionString(expression *BinaryExpression) string
	VisitGroupingExpressionString(expression *GroupingExpression) string
	VisitLiteralExpressionString(expression *LiteralExpression) string
	VisitUnaryExpressionString(expression *UnaryExpression) string
}
