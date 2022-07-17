package main

type Expression interface {
}

type BinaryExpresion struct {
	Left     Expression
	Operator Token
	Right    Expression
}
