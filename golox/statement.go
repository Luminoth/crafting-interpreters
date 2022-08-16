/* This file is autogenerated, DO NOT MODIFY */
package main

type Statement interface {
	Accept(visitor StatementVisitor) (*Value, error)
}

type ExpressionStatement struct {
	Expression Expression
}

func (e *ExpressionStatement) Accept(visitor StatementVisitor) (*Value, error) {
	return visitor.VisitExpressionStatement(e)
}

type FunctionStatement struct {
	Name   *Token
	Params []*Token
	Body   []Statement
}

func (e *FunctionStatement) Accept(visitor StatementVisitor) (*Value, error) {
	return visitor.VisitFunctionStatement(e)
}

type PrintStatement struct {
	Expression Expression
}

func (e *PrintStatement) Accept(visitor StatementVisitor) (*Value, error) {
	return visitor.VisitPrintStatement(e)
}

type ReturnStatement struct {
	Keyword *Token
	Value   Expression
}

func (e *ReturnStatement) Accept(visitor StatementVisitor) (*Value, error) {
	return visitor.VisitReturnStatement(e)
}

type VarStatement struct {
	Name        *Token
	Initializer Expression
}

func (e *VarStatement) Accept(visitor StatementVisitor) (*Value, error) {
	return visitor.VisitVarStatement(e)
}

type BlockStatement struct {
	Statements []Statement
}

func (e *BlockStatement) Accept(visitor StatementVisitor) (*Value, error) {
	return visitor.VisitBlockStatement(e)
}

type IfStatement struct {
	Condition Expression
	Then      Statement
	Else      Statement
}

func (e *IfStatement) Accept(visitor StatementVisitor) (*Value, error) {
	return visitor.VisitIfStatement(e)
}

// For statement desugars to a While statement
type WhileStatement struct {
	Condition Expression
	Body      Statement
}

func (e *WhileStatement) Accept(visitor StatementVisitor) (*Value, error) {
	return visitor.VisitWhileStatement(e)
}

type BreakStatement struct {
	Keyword *Token
}

func (e *BreakStatement) Accept(visitor StatementVisitor) (*Value, error) {
	return visitor.VisitBreakStatement(e)
}

type ContinueStatement struct {
	Keyword *Token
}

func (e *ContinueStatement) Accept(visitor StatementVisitor) (*Value, error) {
	return visitor.VisitContinueStatement(e)
}

type StatementVisitor interface {
	VisitExpressionStatement(statement *ExpressionStatement) (*Value, error)
	VisitFunctionStatement(statement *FunctionStatement) (*Value, error)
	VisitPrintStatement(statement *PrintStatement) (*Value, error)
	VisitReturnStatement(statement *ReturnStatement) (*Value, error)
	VisitVarStatement(statement *VarStatement) (*Value, error)
	VisitBlockStatement(statement *BlockStatement) (*Value, error)
	VisitIfStatement(statement *IfStatement) (*Value, error)
	VisitWhileStatement(statement *WhileStatement) (*Value, error)
	VisitBreakStatement(statement *BreakStatement) (*Value, error)
	VisitContinueStatement(statement *ContinueStatement) (*Value, error)
}
