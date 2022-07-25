package main

type Interpreter struct {
}

func (p *Interpreter) VisitBinaryExpression(expression *BinaryExpression) Value {
	return NewNilValue()
}

func (p *Interpreter) VisitTernaryExpression(expression *TernaryExpression) Value {
	return NewNilValue()
}

func (p *Interpreter) VisitGroupingExpression(expression *GroupingExpression) Value {
	return NewNilValue()
}

func (p *Interpreter) VisitLiteralExpression(expression *LiteralExpression) Value {
	return NewNilValue()
}

func (p *Interpreter) VisitUnaryExpression(expression *UnaryExpression) Value {
	return NewNilValue()
}
