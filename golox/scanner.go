package main

import "fmt"

type Scanner struct {
	Tokens []Token `json:"tokens"`

	source []rune

	Start   uint `json:"start"`
	Current uint `json:"current"`
	Line    uint `json:"line"`
}

func (s *Scanner) ScanTokens() {
	s.reset()

	for {
		if !s.scanToken() {
			break
		}
	}

	s.Tokens = append(s.Tokens, Token{
		Type: EOF,
		Line: s.Line,
	})
}

func (s *Scanner) reset() {
	s.Tokens = []Token{}

	s.Start = 0
	s.Current = 0
	s.Line = 1
}

func (s *Scanner) scanToken() bool {
	switch ch := s.advance(); ch {
	// single character tokens
	case '(':
		s.addToken(LeftParen)
	case ')':
		s.addToken(RightParen)
	case '{':
		s.addToken(LeftBrace)
	case '}':
		s.addToken(RightBrace)
	case ',':
		s.addToken(Comma)
	case '.':
		s.addToken(Dot)
	case '-':
		s.addToken(Minus)
	case '+':
		s.addToken(Plus)
	case ';':
		s.addToken(Semicolon)
	case '*':
		s.addToken(Star)

	// one or two character tokens
	case '!':
		if s.match('=') {
			s.addToken(BangEqual)
		} else {
			s.addToken(Bang)
		}
	case '=':
		if s.match('=') {
			s.addToken(EqualEqual)
		} else {
			s.addToken(Equal)
		}
	case '<':
		if s.match('=') {
			s.addToken(LessEqual)
		} else {
			s.addToken(Less)
		}
	case '>':
		if s.match('=') {
			s.addToken(GreaterEqual)
		} else {
			s.addToken(Greater)
		}
	case 0:
		return false
	default:
		reportError(s.Line, fmt.Sprintf("Unexpected character '%c'", ch))
	}

	return true
}

func (s *Scanner) advance() rune {
	if s.isAtEnd() {
		return 0
	}

	c := s.source[s.Current]
	s.Current += 1
	return c
}

func (s *Scanner) match(expected rune) bool {
	if s.isAtEnd() {
		return false
	}

	if s.source[s.Current] != expected {
		return false
	}

	s.Current += 1
	return true
}

func (s *Scanner) addToken(tokenType TokenType) {
	s.addTokenLiteral(tokenType, nil)
}

func (s *Scanner) addTokenLiteral(tokenType TokenType, literal interface{}) {
	lexeme := string(s.source[s.Start:s.Current])
	s.Tokens = append(s.Tokens, Token{
		Type:    tokenType,
		Lexeme:  lexeme,
		Literal: literal,
		Line:    s.Line,
	})
}

func (s *Scanner) isAtEnd() bool {
	return s.Current >= uint(len(s.source))
}

func NewScanner(source string) Scanner {
	return Scanner{
		Tokens: []Token{},
		source: []rune(source),
		Line:   1,
	}
}
