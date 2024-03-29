/* This file is autogenerated, DO NOT MODIFY */
package main

type Expression interface {
	AcceptString(visitor ExpressionVisitor[string]) (string, error)
	AcceptValue(visitor ExpressionVisitor[Value]) (Value, error)
}

type AssignExpression struct {
	Name  *Token
	Value Expression
}

func (e *AssignExpression) AcceptString(visitor ExpressionVisitor[string]) (string, error) {
	return visitor.VisitAssignExpression(e)
}

func (e *AssignExpression) AcceptValue(visitor ExpressionVisitor[Value]) (Value, error) {
	return visitor.VisitAssignExpression(e)
}

type BinaryExpression struct {
	Left     Expression
	Operator *Token
	Right    Expression
}

func (e *BinaryExpression) AcceptString(visitor ExpressionVisitor[string]) (string, error) {
	return visitor.VisitBinaryExpression(e)
}

func (e *BinaryExpression) AcceptValue(visitor ExpressionVisitor[Value]) (Value, error) {
	return visitor.VisitBinaryExpression(e)
}

type CallExpression struct {
	Callee    Expression
	Paren     *Token
	Arguments []Expression
}

func (e *CallExpression) AcceptString(visitor ExpressionVisitor[string]) (string, error) {
	return visitor.VisitCallExpression(e)
}

func (e *CallExpression) AcceptValue(visitor ExpressionVisitor[Value]) (Value, error) {
	return visitor.VisitCallExpression(e)
}

type GetExpression struct {
	Object Expression
	Name   *Token
}

func (e *GetExpression) AcceptString(visitor ExpressionVisitor[string]) (string, error) {
	return visitor.VisitGetExpression(e)
}

func (e *GetExpression) AcceptValue(visitor ExpressionVisitor[Value]) (Value, error) {
	return visitor.VisitGetExpression(e)
}

type GroupingExpression struct {
	Expression Expression
}

func (e *GroupingExpression) AcceptString(visitor ExpressionVisitor[string]) (string, error) {
	return visitor.VisitGroupingExpression(e)
}

func (e *GroupingExpression) AcceptValue(visitor ExpressionVisitor[Value]) (Value, error) {
	return visitor.VisitGroupingExpression(e)
}

type LiteralExpression struct {
	Value LiteralValue
}

func (e *LiteralExpression) AcceptString(visitor ExpressionVisitor[string]) (string, error) {
	return visitor.VisitLiteralExpression(e)
}

func (e *LiteralExpression) AcceptValue(visitor ExpressionVisitor[Value]) (Value, error) {
	return visitor.VisitLiteralExpression(e)
}

type LogicalExpression struct {
	Left     Expression
	Operator *Token
	Right    Expression
}

func (e *LogicalExpression) AcceptString(visitor ExpressionVisitor[string]) (string, error) {
	return visitor.VisitLogicalExpression(e)
}

func (e *LogicalExpression) AcceptValue(visitor ExpressionVisitor[Value]) (Value, error) {
	return visitor.VisitLogicalExpression(e)
}

type SetExpression struct {
	Object Expression
	Name   *Token
	Value  Expression
}

func (e *SetExpression) AcceptString(visitor ExpressionVisitor[string]) (string, error) {
	return visitor.VisitSetExpression(e)
}

func (e *SetExpression) AcceptValue(visitor ExpressionVisitor[Value]) (Value, error) {
	return visitor.VisitSetExpression(e)
}

type SuperExpression struct {
	Keyword *Token
	Method  *Token
}

func (e *SuperExpression) AcceptString(visitor ExpressionVisitor[string]) (string, error) {
	return visitor.VisitSuperExpression(e)
}

func (e *SuperExpression) AcceptValue(visitor ExpressionVisitor[Value]) (Value, error) {
	return visitor.VisitSuperExpression(e)
}

type ThisExpression struct {
	Keyword *Token
}

func (e *ThisExpression) AcceptString(visitor ExpressionVisitor[string]) (string, error) {
	return visitor.VisitThisExpression(e)
}

func (e *ThisExpression) AcceptValue(visitor ExpressionVisitor[Value]) (Value, error) {
	return visitor.VisitThisExpression(e)
}

type TernaryExpression struct {
	Condition Expression
	True      Expression
	False     Expression
}

func (e *TernaryExpression) AcceptString(visitor ExpressionVisitor[string]) (string, error) {
	return visitor.VisitTernaryExpression(e)
}

func (e *TernaryExpression) AcceptValue(visitor ExpressionVisitor[Value]) (Value, error) {
	return visitor.VisitTernaryExpression(e)
}

type UnaryExpression struct {
	Operator *Token
	Right    Expression
}

func (e *UnaryExpression) AcceptString(visitor ExpressionVisitor[string]) (string, error) {
	return visitor.VisitUnaryExpression(e)
}

func (e *UnaryExpression) AcceptValue(visitor ExpressionVisitor[Value]) (Value, error) {
	return visitor.VisitUnaryExpression(e)
}

type VariableExpression struct {
	Name *Token
}

func (e *VariableExpression) AcceptString(visitor ExpressionVisitor[string]) (string, error) {
	return visitor.VisitVariableExpression(e)
}

func (e *VariableExpression) AcceptValue(visitor ExpressionVisitor[Value]) (Value, error) {
	return visitor.VisitVariableExpression(e)
}

type ExpressionVisitorConstraint interface {
	string | Value
}

type ExpressionVisitor[T ExpressionVisitorConstraint] interface {
	VisitAssignExpression(expression *AssignExpression) (T, error)
	VisitBinaryExpression(expression *BinaryExpression) (T, error)
	VisitCallExpression(expression *CallExpression) (T, error)
	VisitGetExpression(expression *GetExpression) (T, error)
	VisitGroupingExpression(expression *GroupingExpression) (T, error)
	VisitLiteralExpression(expression *LiteralExpression) (T, error)
	VisitLogicalExpression(expression *LogicalExpression) (T, error)
	VisitSetExpression(expression *SetExpression) (T, error)
	VisitSuperExpression(expression *SuperExpression) (T, error)
	VisitThisExpression(expression *ThisExpression) (T, error)
	VisitTernaryExpression(expression *TernaryExpression) (T, error)
	VisitUnaryExpression(expression *UnaryExpression) (T, error)
	VisitVariableExpression(expression *VariableExpression) (T, error)
}
