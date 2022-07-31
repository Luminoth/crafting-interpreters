package main

import (
	"fmt"
)

type ParserError struct {
	Message string `json:"message"`

	Tokens []*Token `json:"tokens"`
}

func (e *ParserError) Error() string {
	return e.Message
}

type Parser struct {
	Tokens []*Token `json:"tokens"`

	Current uint `json:"current"`
}

func NewParser(tokens []*Token) Parser {
	return Parser{
		Tokens: tokens,
	}
}

func (p *Parser) ParseProgram() (statements []Statement) {
	for {
		if p.isAtEnd() {
			break
		}

		statement := p.declaration()
		// TODO: statement can be nil here but we aren't handling it ...
		statements = append(statements, statement)
	}

	return
}

func (p *Parser) ParseExpression() (expr Expression) {
	expr, err := p.expression()
	if err != nil {
		return nil
	}

	if !p.isAtEnd() {
		p.error(p.peek(), "Expected end of file.")
		return nil
	}

	return
}

func (p *Parser) declaration() (statement Statement) {
	var err error

	if p.match(Var) {
		statement, err = p.variableDeclaration()
	} else {
		statement, err = p.statement()
	}

	if err != nil {
		p.synchronize()
		return nil
	}

	return statement
}

func (p *Parser) variableDeclaration() (statement Statement, err error) {
	name, err := p.consume(Identifier, "Expect variable name.")
	if err != nil {
		return
	}

	var initializer Expression
	if p.match(Equal) {
		initializer, err = p.expression()
		if err != nil {
			return
		}
	}

	_, err = p.consume(Semicolon, "Expect ';' after variable declaration.")
	//_, err = p.consumeSafe(Semicolon)
	if err != nil {
		return
	}

	statement = &VarStatement{
		Name:        name,
		Initializer: initializer,
	}
	return
}

func (p *Parser) statement() (Statement, error) {
	if p.match(Print) {
		return p.printStatement()
	}
	return p.expressionStatement()
}

func (p *Parser) printStatement() (statement Statement, err error) {
	value, err := p.expression()
	if err != nil {
		return
	}

	_, err = p.consume(Semicolon, "Expect ';' after value.")
	//_, err = p.consumeSafe(Semicolon)
	if err != nil {
		return
	}

	statement = &PrintStatement{
		Expression: value,
	}
	return
}

func (p *Parser) expressionStatement() (statement Statement, err error) {
	value, err := p.expression()
	if err != nil {
		return
	}

	_, err = p.consume(Semicolon, "Expect ';' after expression.")
	//_, err = p.consumeSafe(Semicolon)
	if err != nil {
		return
	}

	statement = &ExpressionStatement{
		Expression: value,
	}
	return
}

func (p *Parser) expression() (Expression, error) {
	return p.comma()
}

func (p *Parser) binaryExpression(operand func() (Expression, error), tokenTypes ...TokenType) (expr Expression, err error) {
	expr, err = operand()
	if err != nil {
		return
	}

	for {
		if !p.match(tokenTypes...) {
			break
		}

		operator := p.previous()

		right, innerErr := operand()
		if innerErr != nil {
			err = innerErr
			return
		}

		expr = &BinaryExpression{
			Left:     expr,
			Operator: operator,
			Right:    right,
		}
	}
	return
}

func (p *Parser) comma() (Expression, error) {
	return p.binaryExpression(p.assignment, Comma)
}

func (p *Parser) assignment() (expr Expression, err error) {
	expr, err = p.ternary()
	if err != nil {
		return
	}

	if p.match(Equal) {
		equals := p.previous()
		value, innerErr := p.assignment()
		if innerErr != nil {
			err = innerErr
			return
		}

		if v, ok := expr.(*VariableExpression); ok {
			expr = &AssignExpression{
				Name:  v.Name,
				Value: value,
			}
			return
		}

		p.error(equals, "Invalid assignment target.")
	}

	return
}

func (p *Parser) ternary() (expr Expression, err error) {
	expr, err = p.equality()
	if err != nil {
		return
	}

	if !p.match(Question) {
		return
	}

	left, err := p.expression()
	if err != nil {
		return
	}

	_, err = p.consume(Colon, "Expect ':' after expression.")
	if err != nil {
		return
	}

	right, err := p.ternary()
	if err != nil {
		return
	}

	expr = &TernaryExpression{
		Condition: expr,
		True:      left,
		False:     right,
	}
	return
}

func (p *Parser) equality() (Expression, error) {
	return p.binaryExpression(p.comparison, BangEqual, EqualEqual)
}

func (p *Parser) comparison() (Expression, error) {
	return p.binaryExpression(p.term, Greater, GreaterEqual, Less, LessEqual)
}

func (p *Parser) term() (Expression, error) {
	return p.binaryExpression(p.factor, Minus, Plus)
}

func (p *Parser) factor() (expr Expression, err error) {
	return p.binaryExpression(p.unary, Slash, Star)
}

func (p *Parser) unary() (expr Expression, err error) {
	if !p.match(Bang, Minus) {
		return p.primary()
	}

	operator := p.previous()

	right, err := p.unary()
	if err != nil {
		return
	}

	expr = &UnaryExpression{
		Operator: operator,
		Right:    right,
	}
	return
}

func (p *Parser) primary() (expr Expression, err error) {
	if p.match(False) {
		expr = &LiteralExpression{
			Value: NewBoolLiteral(false),
		}
		return
	}

	if p.match(True) {
		expr = &LiteralExpression{
			Value: NewBoolLiteral(true),
		}
		return
	}

	if p.match(Nil) {
		expr = &LiteralExpression{
			Value: NewNilLiteral(),
		}
		return
	}

	if p.match(Number) {
		expr = &LiteralExpression{
			Value: NewNumberLiteral(p.previous().Literal.NumberValue),
		}
		return
	}

	if p.match(String) {
		expr = &LiteralExpression{
			Value: NewStringLiteral(p.previous().Literal.StringValue),
		}
		return
	}

	if p.match(Identifier) {
		expr = &VariableExpression{
			Name: p.previous(),
		}
		return
	}

	if p.match(LeftParen) {
		expression, innerErr := p.expression()
		if innerErr != nil {
			err = innerErr
			return
		}

		_, err = p.consume(RightParen, "Expect ')' after expression.")
		if err != nil {
			return
		}

		expr = &GroupingExpression{
			Expression: expression,
		}
		return
	}

	err = p.error(p.peek(), "Expect expression.")
	return
}

func (p *Parser) peek() *Token {
	return p.Tokens[p.Current]
}

func (p *Parser) check(tokenType TokenType) bool {
	if p.isAtEnd() {
		return false
	}
	return p.peek().Type == tokenType
}

func (p *Parser) match(tokenTypes ...TokenType) bool {
	for _, tokenType := range tokenTypes {
		if p.check(tokenType) {
			p.advance()
			return true
		}
	}
	return false
}

func (p *Parser) consume(tokenType TokenType, message string) (token *Token, err error) {
	if p.check(tokenType) {
		token = p.advance()
		return
	}

	err = p.error(p.peek(), message)
	return
}

/*func (p *Parser) consumeSafe(tokenType TokenType) (token *Token, err error) {
	if p.check(tokenType) {
		token = p.advance()
		return
	}

	// didn't find the token, so insert it
	p.Tokens = Insert(p.Tokens, int(p.Current), &Token{
		Type: tokenType,
		// TODO: can we fill in anything else?
	})

	token = p.advance()
	return
}*/

func (p *Parser) advance() *Token {
	if !p.isAtEnd() {
		p.Current++
	}
	return p.previous()
}

func (p *Parser) previous() *Token {
	return p.Tokens[p.Current-1]
}

func (p *Parser) isAtEnd() bool {
	return p.peek().Type == EOF
}

func (p *Parser) error(token *Token, message string) error {
	if token.Type == EOF {
		report(token.Line, " at end", message)
	} else {
		report(token.Line, fmt.Sprintf(" at '%s'", token.Lexeme), message)
	}

	return &ParserError{
		Message: message,
		Tokens:  []*Token{token},
	}
}

func (p *Parser) synchronize() {
	p.advance()
	for {
		if p.isAtEnd() {
			break
		}

		// did we just end an expression?
		if p.previous().Type == Semicolon {
			return
		}

		// are we at the start of a new statement?
		switch p.peek().Type {
		case Class:
			fallthrough
		case For:
			fallthrough
		case Fun:
			fallthrough
		case If:
			fallthrough
		case Print:
			fallthrough
		case Return:
			fallthrough
		case Var:
			fallthrough
		case While:
			return
		}

		p.advance()
	}
}
