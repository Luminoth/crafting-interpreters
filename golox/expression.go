package main

type Expression interface {
}

type BinaryExpression struct {
	Left     Expression
	Operator Token
	Right    Expression
}

type GroupingExpression struct {
	Expression Expression
}

type LiteralExpression struct {
	Value interface{}
}

type UnaryExpression struct {
	Operator Token
	Right    Expression
}

type ExpressionVisitor[T any] interface {
	VisitBinaryExpression(expression *BinaryExpression) T
	VisitGroupingExpression(expression *GroupingExpression) T
	VisitLiteralExpression(expression *LiteralExpression) T
	VisitUnaryExpression(expression *UnaryExpression) T
}

type ExpressionVisitorFacilitator[T any] interface {
	Accept(visitor ExpressionVisitor[T]) T
}

type BinaryExpressionAcceptor[T any] struct {
	Expression *BinaryExpression
}

func NewBinaryExpressionAcceptor[T any](expression *BinaryExpression) *BinaryExpressionAcceptor[T] {
	return &BinaryExpressionAcceptor[T]{
		Expression: expression,
	}
}

func (a *BinaryExpressionAcceptor[T]) Accept(visitor ExpressionVisitor[T]) T {
	return visitor.VisitBinaryExpression(a.Expression)
}

type GroupingExpressionAcceptor[T any] struct {
	Expression *GroupingExpression
}

func NewGroupingExpressionAcceptor[T any](expression *GroupingExpression) *GroupingExpressionAcceptor[T] {
	return &GroupingExpressionAcceptor[T]{
		Expression: expression,
	}
}

func (a *GroupingExpressionAcceptor[T]) Accept(visitor ExpressionVisitor[T]) T {
	return visitor.VisitGroupingExpression(a.Expression)
}

type LiteralExpressionAcceptor[T any] struct {
	Expression *LiteralExpression
}

func NewLiteralExpressionAcceptor[T any](expression *LiteralExpression) *LiteralExpressionAcceptor[T] {
	return &LiteralExpressionAcceptor[T]{
		Expression: expression,
	}
}

func (a *LiteralExpressionAcceptor[T]) Accept(visitor ExpressionVisitor[T]) T {
	return visitor.VisitLiteralExpression(a.Expression)
}

type UnaryExpressionAcceptor[T any] struct {
	Expression *UnaryExpression
}

func NewUnaryExpressionAcceptor[T any](expression *UnaryExpression) *UnaryExpressionAcceptor[T] {
	return &UnaryExpressionAcceptor[T]{
		Expression: expression,
	}
}

func (a *UnaryExpressionAcceptor[T]) Accept(visitor ExpressionVisitor[T]) T {
	return visitor.VisitUnaryExpression(a.Expression)
}
