package main

import (
	"fmt"
	"strconv"
	"unicode"
)

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
		s.Start = s.Current
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

	// special handling for slash (division and comments)
	case '/':
		if s.match('/') {
			for {
				ch := s.peek()
				if ch == '\n' || ch == 0 {
					break
				}
				s.advance()
			}
		} else {
			s.addToken(Slash)
		}

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

	// literals
	case '"':
		s.stringLiteral()

	// ignore whitespace
	case ' ', '\r', '\t':

	// line counter
	case '\n':
		s.Line++

	// EOF
	case 0:
		return false

	default:
		// number literals
		if unicode.IsDigit(ch) {
			s.numberLiteral()
		} else {
			reportError(s.Line, fmt.Sprintf("Unexpected character '%c'", ch))
		}
	}

	return true
}

func (s *Scanner) peek() rune {
	if s.isAtEnd() {
		return 0
	}
	return s.source[s.Current]
}

func (s *Scanner) peekNext() rune {
	if int(s.Current+1) >= len(s.source) {
		return 0
	}
	return s.source[s.Current+1]
}

func (s *Scanner) advance() rune {
	if s.isAtEnd() {
		return 0
	}

	c := s.source[s.Current]
	s.Current++
	return c
}

func (s *Scanner) match(expected rune) bool {
	if s.isAtEnd() {
		return false
	}

	if s.source[s.Current] != expected {
		return false
	}

	s.Current++
	return true
}

func (s *Scanner) stringLiteral() {
	for {
		ch := s.peek()
		if ch == '"' || ch == 0 {
			break
		}

		// allow multiline strings
		if ch == '\n' {
			s.Line++
		}

		s.advance()
	}

	if s.isAtEnd() {
		reportError(s.Line, fmt.Sprintf("Unterminated string literal '%s'", s.lexeme()))
		return
	}

	// consume the closing '"'
	s.advance()

	// trim the quotes from the value
	value := string(s.source[s.Start+1 : s.Current-1])
	s.addTokenLiteral(String, value)
}

func (s *Scanner) numberLiteral() {
	for {
		ch := s.peek()
		if !unicode.IsDigit(ch) {
			break
		}
		s.advance()
	}

	// check for fractional
	if s.peek() == '.' && unicode.IsDigit(s.peekNext()) {
		// consume the '.'
		s.advance()

		for {
			ch := s.peek()
			if !unicode.IsDigit(ch) {
				break
			}
			s.advance()
		}
	}

	value, err := strconv.ParseFloat(s.lexeme(), 64)
	if err != nil {
		reportError(s.Line, fmt.Sprintf("Invalid number literal '%s': %s", s.lexeme(), err.Error()))
		return
	}
	s.addTokenLiteral(Number, value)
}

func (s *Scanner) lexeme() string {
	end := Min(len(s.source), int(s.Current))
	return string(s.source[s.Start:end])
}

func (s *Scanner) addToken(tokenType TokenType) {
	s.addTokenLiteral(tokenType, nil)
}

func (s *Scanner) addTokenLiteral(tokenType TokenType, literal interface{}) {
	s.Tokens = append(s.Tokens, Token{
		Type:    tokenType,
		Lexeme:  s.lexeme(),
		Literal: literal,
		Line:    s.Line,
	})
}

func (s *Scanner) isAtEnd() bool {
	return int(s.Current) >= len(s.source)
}

func NewScanner(source string) Scanner {
	return Scanner{
		Tokens: []Token{},
		source: []rune(source),
		Line:   1,
	}
}
