package main

type Expression interface {
	Accept(visitor ExpressionVisitor) interface{}
}

type BinaryExpression struct {
	Left     Expression
	Operator *Token
	Right    Expression
}

func (e *BinaryExpression) Accept(visitor ExpressionVisitor) interface{} {
	return visitor.VisitBinaryExpression(e)
}

type GroupingExpression struct {
	Expression Expression
}

func (e *GroupingExpression) Accept(visitor ExpressionVisitor) interface{} {
	return visitor.VisitGroupingExpression(e)
}

type LiteralExpression struct {
	Value LiteralValue
}

func (e *LiteralExpression) Accept(visitor ExpressionVisitor) interface{} {
	return visitor.VisitLiteralExpression(e)
}

type UnaryExpression struct {
	Operator *Token
	Right    Expression
}

func (e *UnaryExpression) Accept(visitor ExpressionVisitor) interface{} {
	return visitor.VisitUnaryExpression(e)
}

type ExpressionVisitor interface {
	VisitBinaryExpression(expression *BinaryExpression) interface{}
	VisitGroupingExpression(expression *GroupingExpression) interface{}
	VisitLiteralExpression(expression *LiteralExpression) interface{}
	VisitUnaryExpression(expression *UnaryExpression) interface{}
}
