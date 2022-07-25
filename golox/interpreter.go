package main

import (
	"fmt"
	"os"
)

type Interpreter struct {
}

func (i *Interpreter) VisitBinaryExpression(expression *BinaryExpression) Value {
	left := i.evaluate(expression.Left)
	right := i.evaluate(expression.Right)

	switch expression.Operator.Type {
	case Comma:
		return right
	case Greater:
		return NewBoolValue(left.NumberValue > right.NumberValue)
	case GreaterEqual:
		return NewBoolValue(left.NumberValue >= right.NumberValue)
	case Less:
		return NewBoolValue(left.NumberValue < right.NumberValue)
	case LessEqual:
		return NewBoolValue(left.NumberValue <= right.NumberValue)
	case BangEqual:
		return NewBoolValue(!i.isEqual(left, right))
	case EqualEqual:
		return NewBoolValue(i.isEqual(left, right))
	case Minus:
		return NewNumberValue(left.NumberValue - right.NumberValue)
	case Plus:
		if left.Type == ValueTypeNumber && right.Type == ValueTypeNumber {
			return NewNumberValue(left.NumberValue + right.NumberValue)
		} else if left.Type == ValueTypeString && right.Type == ValueTypeString {
			return NewStringValue(left.StringValue + right.StringValue)
		} else {
			fmt.Printf("Unsupported plus operand types %v, %v", left.Type, right.Type)
			os.Exit(1)
			return NewNilValue()
		}
	case Slash:
		return NewNumberValue(left.NumberValue / right.NumberValue)
	case Star:
		return NewNumberValue(left.NumberValue * right.NumberValue)
	}

	fmt.Printf("Unsupported binary operator type %v", expression.Operator.Type)
	os.Exit(1)
	return NewNilValue()
}

func (i *Interpreter) VisitTernaryExpression(expression *TernaryExpression) Value {
	condition := i.evaluate(expression.Condition)

	if i.isTruthy(condition) {
		return i.evaluate(expression.True)
	} else {
		return i.evaluate(expression.False)
	}
}

func (i *Interpreter) VisitUnaryExpression(expression *UnaryExpression) Value {
	right := i.evaluate(expression.Right)

	switch expression.Operator.Type {
	case Minus:
		return NewNumberValue(-right.NumberValue)
	case Bang:
		return NewBoolValue(!i.isTruthy(right))
	}

	fmt.Printf("Unsupported unary operator type %v", expression.Operator.Type)
	os.Exit(1)
	return NewNilValue()
}

func (i *Interpreter) VisitGroupingExpression(expression *GroupingExpression) Value {
	return i.evaluate(expression.Expression)
}

func (i *Interpreter) VisitLiteralExpression(expression *LiteralExpression) Value {
	return NewValue(expression.Value)
}

func (i *Interpreter) isEqual(left Value, right Value) bool {
	switch left.Type {
	case ValueTypeNil:
		return right.Type == ValueTypeNil
	case ValueTypeAny:
		if right.Type == ValueTypeAny {
			return left.AnyValue == right.AnyValue
		}
		return false
	case ValueTypeString:
		if right.Type == ValueTypeString {
			return left.StringValue == right.StringValue
		}
		return false
	case ValueTypeNumber:
		if right.Type == ValueTypeNumber {
			return left.NumberValue == right.NumberValue
		}
		return false
	case ValueTypeBool:
		if right.Type == ValueTypeBool {
			return left.BoolValue == right.BoolValue
		}
		return false
	}

	fmt.Printf("Unsupported equality value type %v", left.Type)
	os.Exit(1)
	return false
}

func (i *Interpreter) isTruthy(value Value) bool {
	switch value.Type {
	case ValueTypeNil:
		return false
	case ValueTypeAny:
		return true
	case ValueTypeString:
		return true
	case ValueTypeNumber:
		return true
	case ValueTypeBool:
		return value.BoolValue
	}

	fmt.Printf("Unsupported truthy value type %v", value.Type)
	os.Exit(1)
	return false
}

func (i *Interpreter) evaluate(expression Expression) Value {
	return expression.AcceptValue(i)
}
