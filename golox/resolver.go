package main

import "fmt"

type Resolver struct {
	Interpreter *Interpreter `json:"interpreter"`

	// each scope is name => have we finished resolving this variable's initializer yet?
	Scopes Stack[map[string]bool] `json:"scopes"`

	Debug bool `json:"debug"`
}

func NewResolver(interpreter *Interpreter) Resolver {
	return Resolver{
		Interpreter: interpreter,
		Debug:       interpreter.Debug,
	}
}

func (r *Resolver) Resolve(statements []Statement) {
	if r.Debug {
		fmt.Println("Running resolver ...")
	}

	r.resolveStatements(statements)

	// TODO: handle errors
}

func (r *Resolver) beginScope() {
	r.Scopes.Push(map[string]bool{})
}

func (r *Resolver) endScope() {
	r.Scopes.Pop()
}

func (r *Resolver) declare(name *Token) {
	// global scope not tracked
	if r.Scopes.IsEmpty() {
		return
	}

	scope, _ := r.Scopes.Peek()
	scope[name.Lexeme] = false
}

func (r *Resolver) define(name *Token) {
	// global scope not tracked
	if r.Scopes.IsEmpty() {
		return
	}

	scope, _ := r.Scopes.Peek()
	scope[name.Lexeme] = true
}

func (r *Resolver) resolveLocal(expression Expression, name *Token) {
	for idx, scope := range r.Scopes {
		if _, ok := scope[name.Lexeme]; ok {
			r.Interpreter.Resolve(expression, r.Scopes.Size()-1-idx)
			return
		}
	}
	// assume global if we didn't find it
}

func (r *Resolver) VisitExpressionStatement(statement *ExpressionStatement) (value *Value, err error) {
	err = r.resolveExpression(statement.Expression)
	if err != nil {
		return
	}

	return
}

func (r *Resolver) VisitFunctionStatement(statement *FunctionStatement) (value *Value, err error) {
	r.declare(statement.Name)
	r.define(statement.Name)

	err = r.resolveFunction(statement)
	if err != nil {
		return
	}

	return
}

func (r *Resolver) VisitPrintStatement(statement *PrintStatement) (value *Value, err error) {
	err = r.resolveExpression(statement.Expression)
	if err != nil {
		return
	}

	return
}

func (r *Resolver) VisitReturnStatement(statement *ReturnStatement) (value *Value, err error) {
	if statement.Value != nil {
		err = r.resolveExpression(statement.Value)
		if err != nil {
			return
		}
	}

	return
}

func (r *Resolver) VisitBlockStatement(statement *BlockStatement) (value *Value, err error) {
	r.beginScope()
	err = r.resolveStatements(statement.Statements)
	if err != nil {
		return
	}
	r.endScope()

	return
}

func (r *Resolver) VisitIfStatement(statement *IfStatement) (value *Value, err error) {
	err = r.resolveExpression(statement.Condition)
	if err != nil {
		return
	}

	err = r.resolveStatement(statement.Then)
	if err != nil {
		return
	}

	if statement.Else != nil {
		err = r.resolveStatement(statement.Else)
		if err != nil {
			return
		}
	}

	return
}

func (r *Resolver) VisitVarStatement(statement *VarStatement) (value *Value, err error) {
	r.declare(statement.Name)
	if statement.Initializer != nil {
		err = r.resolveExpression(statement.Initializer)
		if err != nil {
			return
		}
	}
	r.define(statement.Name)

	return
}

func (r *Resolver) VisitWhileStatement(statement *WhileStatement) (value *Value, err error) {
	err = r.resolveExpression(statement.Condition)
	if err != nil {
		return
	}

	err = r.resolveStatement(statement.Body)
	if err != nil {
		return
	}

	return
}

func (r *Resolver) VisitBreakStatement(statement *BreakStatement) (value *Value, err error) {
	return
}

func (r *Resolver) VisitContinueStatement(statement *ContinueStatement) (value *Value, err error) {
	return
}

func (r *Resolver) resolveStatements(statements []Statement) error {
	for _, statement := range statements {
		err := r.resolveStatement(statement)
		if err != nil {
			return err
		}
	}
	return nil
}

func (r *Resolver) resolveStatement(statement Statement) error {
	_, err := statement.Accept(r)
	return err
}

func (r *Resolver) resolveFunction(function *FunctionStatement) (err error) {
	r.beginScope()

	for _, param := range function.Params {
		r.declare(param)
		r.define(param)
	}

	err = r.resolveStatements(function.Body)
	if err != nil {
		return err
	}

	r.endScope()
	return
}

func (r *Resolver) VisitAssignExpression(expression *AssignExpression) (value Value, err error) {
	err = r.resolveExpression(expression)
	if err != nil {
		return
	}

	r.resolveLocal(expression, expression.Name)
	return
}

func (r *Resolver) VisitBinaryExpression(expression *BinaryExpression) (value Value, err error) {
	err = r.resolveExpression(expression.Left)
	if err != nil {
		return
	}

	err = r.resolveExpression(expression.Right)
	if err != nil {
		return
	}

	return
}

func (r *Resolver) VisitTernaryExpression(expression *TernaryExpression) (value Value, err error) {
	err = r.resolveExpression(expression.Condition)
	if err != nil {
		return
	}

	err = r.resolveExpression(expression.True)
	if err != nil {
		return
	}

	err = r.resolveExpression(expression.False)
	if err != nil {
		return
	}

	return
}

func (r *Resolver) VisitLogicalExpression(expression *LogicalExpression) (value Value, err error) {
	err = r.resolveExpression(expression.Left)
	if err != nil {
		return
	}

	err = r.resolveExpression(expression.Right)
	if err != nil {
		return
	}

	return
}

func (r *Resolver) VisitUnaryExpression(expression *UnaryExpression) (value Value, err error) {
	err = r.resolveExpression(expression.Right)
	if err != nil {
		return
	}

	return
}

func (r *Resolver) VisitCallExpression(expression *CallExpression) (value Value, err error) {
	err = r.resolveExpression(expression.Callee)
	if err != nil {
		return
	}

	for _, argument := range expression.Arguments {
		err = r.resolveExpression(argument)
		if err != nil {
			return
		}
	}

	return
}

func (r *Resolver) VisitGroupingExpression(expression *GroupingExpression) (value Value, err error) {
	err = r.resolveExpression(expression.Expression)
	if err != nil {
		return
	}

	return
}

func (r *Resolver) VisitLiteralExpression(expression *LiteralExpression) (value Value, err error) {
	return
}

func (r *Resolver) VisitVariableExpression(expression *VariableExpression) (value Value, err error) {
	if !r.Scopes.IsEmpty() {
		// is the variable being accessed from its own initializer?
		// (declared but not defined)
		scope, _ := r.Scopes.Peek()
		v, ok := scope[expression.Name.Lexeme]
		if ok && !v {
			reportError(expression.Name, "Can't read local variable in its own initializer.")
		}
	}

	r.resolveLocal(expression, expression.Name)
	return
}

func (r *Resolver) resolveExpression(expression Expression) error {
	_, err := expression.AcceptValue(r)
	return err
}
