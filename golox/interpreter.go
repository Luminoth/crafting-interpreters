package main

import (
	"fmt"
)

type InterpreterError struct {
	Message string `json:"message"`

	Token *Token `json:"tokens"`
}

func (e *InterpreterError) Error() string {
	return e.Message
}

type Interpreter struct {
}

func (i *Interpreter) VisitBinaryExpression(expression *BinaryExpression) (value Value, err error) {
	left, err := i.evaluate(expression.Left)
	if err != nil {
		return
	}

	right, err := i.evaluate(expression.Right)
	if err != nil {
		return
	}

	switch expression.Operator.Type {
	case Comma:
		value = right
	case Greater:
		err = i.checkNumberOperands(expression.Operator, left, right)
		if err != nil {
			return
		}
		value = NewBoolValue(left.NumberValue > right.NumberValue)
	case GreaterEqual:
		err = i.checkNumberOperands(expression.Operator, left, right)
		if err != nil {
			return
		}
		value = NewBoolValue(left.NumberValue >= right.NumberValue)
	case Less:
		err = i.checkNumberOperands(expression.Operator, left, right)
		if err != nil {
			return
		}
		value = NewBoolValue(left.NumberValue < right.NumberValue)
	case LessEqual:
		err = i.checkNumberOperands(expression.Operator, left, right)
		if err != nil {
			return
		}
		value = NewBoolValue(left.NumberValue <= right.NumberValue)
	case BangEqual:
		isEqual, innerErr := i.isEqual(left, right)
		if innerErr != nil {
			err = innerErr
			return
		}
		value = NewBoolValue(!isEqual)
	case EqualEqual:
		isEqual, innerErr := i.isEqual(left, right)
		if innerErr != nil {
			err = innerErr
			return
		}
		value = NewBoolValue(isEqual)
	case Minus:
		err = i.checkNumberOperands(expression.Operator, left, right)
		if err != nil {
			return
		}
		value = NewNumberValue(left.NumberValue - right.NumberValue)
	case Plus:
		if left.Type == ValueTypeNumber && right.Type == ValueTypeNumber {
			value = NewNumberValue(left.NumberValue + right.NumberValue)
		} else if left.Type == ValueTypeString && right.Type == ValueTypeString {
			value = NewStringValue(left.StringValue + right.StringValue)
		} else {
			err = &InterpreterError{
				Message: "Operands must be two numbers or two strings.",
				Token:   expression.Operator,
			}
		}
	case Slash:
		err = i.checkNumberOperands(expression.Operator, left, right)
		if err != nil {
			return
		}
		value = NewNumberValue(left.NumberValue / right.NumberValue)
	case Star:
		err = i.checkNumberOperands(expression.Operator, left, right)
		if err != nil {
			return
		}
		value = NewNumberValue(left.NumberValue * right.NumberValue)
	default:
		err = fmt.Errorf("unsupported binary operator type %v", expression.Operator.Type)
	}

	return
}

func (i *Interpreter) VisitTernaryExpression(expression *TernaryExpression) (value Value, err error) {
	condition, err := i.evaluate(expression.Condition)
	if err != nil {
		return
	}

	isTruthy, err := i.isTruthy(condition)
	if err != nil {
		return
	}

	if isTruthy {
		return i.evaluate(expression.True)
	} else {
		return i.evaluate(expression.False)
	}
}

func (i *Interpreter) VisitUnaryExpression(expression *UnaryExpression) (value Value, err error) {
	right, err := i.evaluate(expression.Right)
	if err != nil {
		return
	}

	switch expression.Operator.Type {
	case Minus:
		err = i.checkNumberOperands(expression.Operator, right)
		if err != nil {
			return
		}
		value = NewNumberValue(-right.NumberValue)
	case Bang:
		isTruthy, innerErr := i.isTruthy(right)
		if innerErr != nil {
			err = innerErr
			return
		}
		value = NewBoolValue(!isTruthy)
	default:
		err = fmt.Errorf("unsupported unary operator type %v", expression.Operator.Type)
	}

	return
}

func (i *Interpreter) VisitGroupingExpression(expression *GroupingExpression) (value Value, err error) {
	return i.evaluate(expression.Expression)
}

func (i *Interpreter) VisitLiteralExpression(expression *LiteralExpression) (value Value, err error) {
	return NewValue(expression.Value)
}

func (i *Interpreter) isEqual(left Value, right Value) (ok bool, err error) {
	switch left.Type {
	case ValueTypeNil:
		ok = right.Type == ValueTypeNil
	case ValueTypeAny:
		if right.Type == ValueTypeAny {
			ok = left.AnyValue == right.AnyValue
		} else {
			ok = false
		}
	case ValueTypeString:
		if right.Type == ValueTypeString {
			ok = left.StringValue == right.StringValue
		} else {
			ok = false
		}
	case ValueTypeNumber:
		if right.Type == ValueTypeNumber {
			ok = left.NumberValue == right.NumberValue
		} else {
			ok = false
		}
	case ValueTypeBool:
		if right.Type == ValueTypeBool {
			ok = left.BoolValue == right.BoolValue
		} else {
			ok = false
		}
	default:
		err = fmt.Errorf("unsupported equality value type %v", left.Type)
	}

	return
}

func (i *Interpreter) isTruthy(value Value) (ok bool, err error) {
	switch value.Type {
	case ValueTypeNil:
		ok = false
	case ValueTypeAny:
		ok = true
	case ValueTypeString:
		ok = true
	case ValueTypeNumber:
		ok = true
	case ValueTypeBool:
		ok = value.BoolValue
	default:
		err = fmt.Errorf("unsupported truthy value type %v", value.Type)
	}

	return
}

func (i *Interpreter) evaluate(expression Expression) (value Value, err error) {
	return expression.AcceptValue(i)
}

func (i *Interpreter) checkNumberOperands(operator *Token, values ...Value) error {
	for _, value := range values {
		if value.Type != ValueTypeNumber {
			return &InterpreterError{
				Message: "Operand must be a number.",
				Token:   operator,
			}
		}
	}
	return nil
}
