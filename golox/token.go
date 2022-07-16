package main

type TokenType int

const (
	// single character tokens
	LEFT_PAREN  TokenType = 0
	RIGHT_PAREN TokenType = 1
	LEFT_BRACE  TokenType = 2
	RIGHT_BRACE TokenType = 3
	COMMA       TokenType = 4
	DOT         TokenType = 5
	MINUS       TokenType = 6
	PLUS        TokenType = 7
	SEMICOLON   TokenType = 8
	SLASH       TokenType = 9
	STAR        TokenType = 10

	// one or two character tokens
	BANG          TokenType = 11
	BANG_EQUAL    TokenType = 12
	EQUAL         TokenType = 13
	EQUAL_EQUAL   TokenType = 14
	GREATER       TokenType = 15
	GREATER_EQUAL TokenType = 16
	LESS          TokenType = 17
	LESS_EQUAL    TokenType = 18

	// literals
	IDENTIFIER TokenType = 19
	STRING     TokenType = 20
	NUMBER     TokenType = 21

	// keywords
	AND    TokenType = 22
	CLASS  TokenType = 23
	ELSE   TokenType = 24
	FALSE  TokenType = 25
	FUN    TokenType = 26
	FOR    TokenType = 27
	IF     TokenType = 28
	NIL    TokenType = 29
	OR     TokenType = 30
	PRINT  TokenType = 31
	RETURN TokenType = 32
	SUPER  TokenType = 33
	THIS   TokenType = 34
	TRUE   TokenType = 35
	VAR    TokenType = 36
	WHILE  TokenType = 37

	EOF TokenType = -1
)

type Token struct {
	Type    TokenType   `json:"type"`
	Lexeme  string      `json:"lexeme"`
	Literal interface{} `json:"literal"`
	Line    int         `json:"line"`
}
