package main

import (
	"fmt"
)

type RuntimeError struct {
	Message string `json:"message"`

	Token *Token `json:"tokens"`
}

func (e *RuntimeError) Error() string {
	return e.Message
}

type ReturnError struct {
	*RuntimeError

	Value *Value `json:"value,omitempty"`
}

func (e *ReturnError) Unwrap() error {
	return e.RuntimeError
}

type BreakError struct {
	*RuntimeError
}

func (e *BreakError) Unwrap() error {
	return e.RuntimeError
}

type ContinueError struct {
	*RuntimeError
}

func (e *ContinueError) Unwrap() error {
	return e.RuntimeError
}

type Interpreter struct {
	Locals map[Expression]int `json:"locals"`

	Globals Environment `json:"globals"`

	// NOTE: an alternative to this is to pass the environment
	// to each Visit() method, letting the stack handle block scope cleanup
	Environment *Environment `json:"environment"`

	Debug bool `json:"debug"`
}

func NewInterpreter(debug bool) Interpreter {
	i := Interpreter{
		Locals:  map[Expression]int{},
		Globals: NewEnvironment(),
		Debug:   debug,
	}

	// define native functions
	clock := &ClockFunction{}
	i.Globals.Define("clock", NewCallableValue("clock", clock.Arity(), clock))

	if printIsNative {
		print := &PrintFunction{}
		i.Globals.Define("print", NewCallableValue("print", print.Arity(), print))
	}

	i.Environment = &i.Globals
	return i
}

func (i *Interpreter) Interpret(statements []Statement) (value *Value) {
	if i.Debug {
		fmt.Println("Running interpreter ...")
	}

	for _, statement := range statements {
		v, err := i.execute(statement)
		if err != nil {
			runtimeError(err)
			return nil
		}
		value = v
	}

	return
}

func (i *Interpreter) Resolve(expression Expression, depth int) {
	i.Locals[expression] = depth
}

func (i *Interpreter) VisitExpressionStatement(statement *ExpressionStatement) (value *Value, err error) {
	v, err := i.evaluate(statement.Expression)
	if err != nil {
		return
	}

	value = &v
	return
}

func (i *Interpreter) VisitFunctionStatement(statement *FunctionStatement) (value *Value, err error) {
	function := NewLoxFunction(statement, i.Environment)
	i.Environment.Define(function.Name(), NewCallableValue(function.Name(), function.Arity(), function))
	return
}

func (i *Interpreter) VisitPrintStatement(statement *PrintStatement) (value *Value, err error) {
	v, err := i.evaluate(statement.Expression)
	if err != nil {
		return
	}

	fmt.Println(v)

	// no return value here
	// because it looks weird to print things twice
	return
}

func (i *Interpreter) VisitReturnStatement(statement *ReturnStatement) (value *Value, err error) {
	var v Value
	if statement.Value != nil {
		v, err = i.evaluate(statement.Value)
		if err != nil {
			return
		}

		value = &v
	}

	err = &ReturnError{
		RuntimeError: &RuntimeError{
			Message: "Return only supported in functions.",
			Token:   statement.Keyword,
		},
		Value: value,
	}
	return
}

func (i *Interpreter) VisitBlockStatement(statement *BlockStatement) (*Value, error) {
	return i.executeBlock(statement.Statements, NewEnvironmentScope(i.Environment))
}

func (i *Interpreter) VisitIfStatement(statement *IfStatement) (value *Value, err error) {
	condition, err := i.evaluate(statement.Condition)
	if err != nil {
		return
	}

	isTruthy, err := i.isTruthy(condition)
	if err != nil {
		return
	}

	if isTruthy {
		return i.execute(statement.Then)
	}

	if statement.Else != nil {
		return i.execute(statement.Else)
	}

	return
}

func (i *Interpreter) VisitVarStatement(statement *VarStatement) (value *Value, err error) {
	var v Value
	if statement.Initializer != nil {
		v, err = i.evaluate(statement.Initializer)
		if err != nil {
			return
		}
	}

	i.Environment.Define(statement.Name.Lexeme, v)
	value = &v
	return
}

func (i *Interpreter) VisitWhileStatement(statement *WhileStatement) (value *Value, err error) {
	for {
		condition, innerErr := i.evaluate(statement.Condition)
		if innerErr != nil {
			err = innerErr
			return
		}

		isTruthy, innerErr := i.isTruthy(condition)
		if innerErr != nil {
			err = innerErr
			return
		}

		if !isTruthy {
			break
		}

		_, innerErr = i.execute(statement.Body)
		if innerErr != nil {
			if _, ok := innerErr.(*BreakError); ok {
				break
			}

			if _, ok := innerErr.(*ContinueError); ok {
				continue
			}

			err = innerErr
			return
		}
	}

	return
}

func (i *Interpreter) VisitBreakStatement(statement *BreakStatement) (value *Value, err error) {
	err = &BreakError{
		RuntimeError: &RuntimeError{
			Message: "Break only supported in loops.",
			Token:   statement.Keyword,
		},
	}
	return
}

func (i *Interpreter) VisitContinueStatement(statement *ContinueStatement) (value *Value, err error) {
	err = &ContinueError{
		RuntimeError: &RuntimeError{
			Message: "Continue only supported in loops.",
			Token:   statement.Keyword,
		},
	}
	return
}

func (i *Interpreter) VisitClassStatement(statement *ClassStatement) (value *Value, err error) {
	// two-stage define / assign so that class methods can reference the class
	i.Environment.Define(statement.Name.Lexeme, Value{})
	class := &LoxClass{
		Name: statement.Name.Lexeme,
	}
	i.Environment.Assign(statement.Name, NewCallableValue(class.Name, class.Arity(), class))
	return
}

func (i *Interpreter) execute(statement Statement) (*Value, error) {
	return statement.Accept(i)
}

func (i *Interpreter) executeBlock(statements []Statement, environment *Environment) (value *Value, err error) {
	previous := i.Environment
	i.Environment = environment
	defer func() {
		i.Environment = previous
	}()

	for _, statement := range statements {
		value, err = i.execute(statement)
		if err != nil {
			return
		}
	}

	return
}

func (i *Interpreter) VisitAssignExpression(expression *AssignExpression) (value Value, err error) {
	value, err = i.evaluate(expression.Value)
	if err != nil {
		return
	}

	if distance, ok := i.Locals[expression]; ok {
		i.Environment.AssignAt(distance, expression.Name, value)
	} else {
		err = i.Globals.Assign(expression.Name, value)
		if err != nil {
			return
		}
	}

	return
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
		} else if left.Type == ValueTypeString {
			if right.Type == ValueTypeString {
				value = NewStringValue(left.StringValue + right.StringValue)
			} else {
				value = NewStringValue(left.StringValue + right.String())
			}
		} else if right.Type == ValueTypeString {
			value = NewStringValue(left.String() + right.StringValue)
		} else {
			err = &RuntimeError{
				Message: "Operands must be two numbers or two strings.",
				Token:   expression.Operator,
			}
		}
	case Slash:
		err = i.checkNumberOperands(expression.Operator, left, right)
		if err != nil {
			return
		}

		if right.NumberValue == 0.0 {
			err = &RuntimeError{
				Message: "Illegal divide by zero.",
				Token:   expression.Operator,
			}
		} else {
			value = NewNumberValue(left.NumberValue / right.NumberValue)
		}
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

func (i *Interpreter) VisitLogicalExpression(expression *LogicalExpression) (value Value, err error) {
	value, err = i.evaluate(expression.Left)
	if err != nil {
		return
	}

	isTruthy, err := i.isTruthy(value)
	if err != nil {
		return
	}

	// short circuit
	switch expression.Operator.Type {
	case Or:
		if isTruthy {
			return
		}
	case And:
		if !isTruthy {
			return
		}
	default:
		err = fmt.Errorf("unsupported logical operator type %v", expression.Operator.Type)
		return
	}

	return i.evaluate(expression.Right)
}

func (i *Interpreter) VisitUnaryExpression(expression *UnaryExpression) (value Value, err error) {
	right, err := i.evaluate(expression.Right)
	if err != nil {
		return
	}

	switch expression.Operator.Type {
	case Minus:
		err = i.checkNumberOperand(expression.Operator, right)
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

func (i *Interpreter) VisitCallExpression(expression *CallExpression) (value Value, err error) {
	callee, err := i.evaluate(expression.Callee)
	if err != nil {
		return
	}

	// TODO: should these be evaluated *after* we do all the validation?
	arguments := make([]Value, len(expression.Arguments))
	for idx, argument := range expression.Arguments {
		argumentValue, innerErr := i.evaluate(argument)
		if innerErr != nil {
			err = innerErr
			return
		}
		arguments[idx] = argumentValue
	}

	if callee.Type != ValueTypeCallable {
		err = &RuntimeError{
			Message: "Can only call functions and classes.",
			Token:   expression.Paren,
		}
		return
	}

	argumentCount := len(arguments)
	if argumentCount != callee.CallableValue.Arity {
		err = &RuntimeError{
			//Message: fmt.Sprintf("'%s' expected %d arguments but got %d.", callee.CallableValue.Name, callee.CallableValue.Arity, argumentCount),
			Message: fmt.Sprintf("Expected %d arguments but got %d.", callee.CallableValue.Arity, argumentCount),
			Token:   expression.Paren,
		}
		return
	}

	v, err := callee.CallableValue.Callable.Call(i, arguments)
	if err != nil {
		return
	}

	if v == nil {
		// TODO: should this be a "void" value?
		value = Value{}
	} else {
		value = *v
	}
	return
}

func (i *Interpreter) VisitGetExpression(expression *GetExpression) (value Value, err error) {
	object, err := i.evaluate(expression.Object)
	if err != nil {
		return
	}

	if object.Type != ValueTypeInstance {
		err = &RuntimeError{
			Message: "Only instances have properties.",
			Token:   expression.Name,
		}
		return
	}

	return object.InstanceValue.Get(expression.Name)
}

func (i *Interpreter) VisitGroupingExpression(expression *GroupingExpression) (Value, error) {
	return i.evaluate(expression.Expression)
}

func (i *Interpreter) VisitLiteralExpression(expression *LiteralExpression) (Value, error) {
	return NewValue(expression.Value)
}

func (i *Interpreter) lookUpVariable(name *Token, expression Expression) (Value, error) {
	if distance, ok := i.Locals[expression]; ok {
		return i.Environment.GetAt(distance, name.Lexeme)
	}
	return i.Globals.Get(name)
}

func (i *Interpreter) VisitVariableExpression(expression *VariableExpression) (Value, error) {
	return i.lookUpVariable(expression.Name, expression)
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

func (i *Interpreter) evaluate(expression Expression) (Value, error) {
	return expression.AcceptValue(i)
}

func (i *Interpreter) checkNumberOperand(operator *Token, value Value) error {
	if value.Type == ValueTypeNumber {
		return nil
	}

	return &RuntimeError{
		//Message: fmt.Sprintf("Operand must be a number (%s).", value),
		Message: "Operand must be a number.",
		Token:   operator,
	}
}

func (i *Interpreter) checkNumberOperands(operator *Token, a Value, b Value) error {
	if a.Type == ValueTypeNumber && b.Type == ValueTypeNumber {
		return nil
	}

	return &RuntimeError{
		//Message: fmt.Sprintf("Operands must be numbers (%s, %s).", a, b),
		Message: "Operands must be numbers.",
		Token:   operator,
	}
}
