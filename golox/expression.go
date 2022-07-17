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
