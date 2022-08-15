package main

type Resolver struct {
	Interpreter *Interpreter `json:"interpreter"`

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

func (r *Resolver) VisitBlockStatement(statement *BlockStatement) (*Value, error) {
	r.beginScope()
	err := r.resolve(statement.Statements)
	if err != nil {
		return nil, err
	}
	r.endScope()

	return nil, nil
}

func (r *Resolver) resolve(statements []Statement) error {
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

func (r *Resolver) resolveExpression(expression Expression) error {
	/*_, err := expression.AcceptValue(r)
	return err*/
	return nil
}
