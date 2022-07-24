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
		Tokens: []*Token{},
	}
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

func (p *Parser) expression() (Expression, error) {
	return p.equality()
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

	err = p.error(p.peek(), "Unexpected primary token.")
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
