package main

import (
	"fmt"
	"os"
)

type Parser struct {
	Tokens []*Token `json:"tokens"`

	Current uint `json:"current"`
}

func NewParser(tokens []*Token) Parser {
	return Parser{
		Tokens: []*Token{},
	}
}

func (p *Parser) expression() Expression {
	return p.equality()
}

func (p *Parser) equality() Expression {
	expression := p.comparison()
	for {
		if !p.match(BangEqual, EqualEqual) {
			break
		}

		operator := p.previous()
		right := p.comparison()
		expression = &BinaryExpression{
			Left:     expression,
			Operator: operator,
			Right:    right,
		}
	}
	return expression
}

func (p *Parser) comparison() Expression {
	expression := p.term()
	for {
		if !p.match(Greater, GreaterEqual, Less, LessEqual) {
			break
		}

		operator := p.previous()
		right := p.term()
		expression = &BinaryExpression{
			Left:     expression,
			Operator: operator,
			Right:    right,
		}
	}
	return expression
}

func (p *Parser) term() Expression {
	expression := p.factor()
	for {
		if !p.match(Minus, Plus) {
			break
		}

		operator := p.previous()
		right := p.factor()
		expression = &BinaryExpression{
			Left:     expression,
			Operator: operator,
			Right:    right,
		}
	}
	return expression
}

func (p *Parser) factor() Expression {
	expression := p.unary()
	for {
		if !p.match(Slash, Star) {
			break
		}

		operator := p.previous()
		right := p.unary()
		expression = &BinaryExpression{
			Left:     expression,
			Operator: operator,
			Right:    right,
		}
	}
	return expression
}

func (p *Parser) unary() Expression {
	if !p.match(Bang, Minus) {
		return p.primary()
	}

	operator := p.previous()
	right := p.unary()
	return &UnaryExpression{
		Operator: operator,
		Right:    right,
	}

}

func (p *Parser) primary() Expression {
	if p.match(False) {
		return &LiteralExpression{
			Value: NewBoolLiteral(false),
		}
	}

	if p.match(True) {
		return &LiteralExpression{
			Value: NewBoolLiteral(true),
		}
	}

	if p.match(Nil) {
		return &LiteralExpression{
			Value: NewNilLiteral(),
		}
	}

	if p.match(Number) {
		return &LiteralExpression{
			Value: NewNumberLiteral(p.previous().Literal.NumberValue),
		}
	}

	if p.match(String) {
		return &LiteralExpression{
			Value: NewStringLiteral(p.previous().Literal.StringValue),
		}
	}

	if p.match(LeftParen) {
		expression := p.expression()
		//p.consume(RightParen, "Expect ')' after expression.")
		return &GroupingExpression{
			Expression: expression,
		}
	}

	fmt.Printf("Unexpected primary token %v", p.peek())
	os.Exit(1)
	return nil
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
