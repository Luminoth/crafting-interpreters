package main

type Resolver struct {
	Interpreter *Interpreter `json:"interpreter"`

	// each scope is name => have we finished resolving this variable's initializer yet?
	Scopes Stack[map[string]bool] `json:"scopes"`
}

func NewResolver(interpreter *Interpreter) Resolver {
	return Resolver{
		Interpreter: interpreter,
	}
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

func (r *Resolver) VisitBlockStatement(statement *BlockStatement) (*Value, error) {
	r.beginScope()
	err := r.resolveStatements(statement.Statements)
	if err != nil {
		return nil, err
	}
	r.endScope()

	return nil, nil
}

func (r *Resolver) VisitVarStatement(statement *VarStatement) (*Value, error) {
	r.declare(statement.Name)
	if statement.Initializer != nil {
		err := r.resolveExpression(statement.Initializer)
		if err != nil {
			return nil, err
		}
	}
	r.define(statement.Name)

	return nil, nil
}

func (r *Resolver) VisitFunctionStatement(statement *FunctionStatement) (*Value, error) {
	r.declare(statement.Name)
	r.define(statement.Name)

	err := r.resolveFunction(statement)
	return nil, err
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
	/*_, err := statement.Accept(r)
	return err*/
	return nil
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

func (r *Resolver) VisitAssignExpression(expression *AssignExpression) (value Value, err error) {
	err = r.resolveExpression(expression)
	if err != nil {
		return
	}

	r.resolveLocal(expression, expression.Name)
	return
}

func (r *Resolver) resolveExpression(expression Expression) error {
	/*_, err := expression.AcceptValue(r)
	return err*/
	return nil
}
