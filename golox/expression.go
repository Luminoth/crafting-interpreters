package main

type Expression interface {
	AcceptString(visitor ExpressionVisitor[string]) string
}

type BinaryExpression struct {
	Left     Expression
	Operator *Token
	Right    Expression
}

func (e *BinaryExpression) AcceptString(visitor ExpressionVisitor[string]) string {
	return visitor.VisitBinaryExpression(e)
}

type GroupingExpression struct {
	Expression Expression
}

func (e *GroupingExpression) AcceptString(visitor ExpressionVisitor[string]) string {
	return visitor.VisitGroupingExpression(e)
}

type LiteralExpression struct {
	Value LiteralValue
}

func (e *LiteralExpression) AcceptString(visitor ExpressionVisitor[string]) string {
	return visitor.VisitLiteralExpression(e)
}

type UnaryExpression struct {
	Operator *Token
	Right    Expression
}

func (e *UnaryExpression) AcceptString(visitor ExpressionVisitor[string]) string {
	return visitor.VisitUnaryExpression(e)
}

type ExpressionVisitor[T any] interface {
	VisitBinaryExpression(expression *BinaryExpression) T
	VisitGroupingExpression(expression *GroupingExpression) T
	VisitLiteralExpression(expression *LiteralExpression) T
	VisitUnaryExpression(expression *UnaryExpression) T
}
