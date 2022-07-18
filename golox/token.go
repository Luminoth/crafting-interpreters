package main

type TokenType uint

// TODO: use stringer to generate this
func (t TokenType) String() string {
	return [...]string{
		"EOF",

		"LeftParen", "RightParen",
		"LeftBrace", "RightBrace",
		"Comma", "Dot",
		"Minus", "Plus",
		"Semicolon", "Slash", "Star",

		"Bang", "BangEqual",
		"Equal", "EqualEqual",
		"Greater", "GreaterEqual",
		"Less", "LessEqual",

		"Identifier", "String", "Number",

		"And", "Or",
		"If", "Else",
		"Class", "Super", "This",
		"True", "False",
		"Fun",
		"For", "While",
		"Nil",
		"Print",
		"Return",
		"Var",
	}[t]
}

const (
	EOF TokenType = 0

	// single character tokens
	LeftParen  TokenType = 1
	RightParen TokenType = 2
	LeftBrace  TokenType = 3
	RightBrace TokenType = 4
	Comma      TokenType = 5
	Dot        TokenType = 6
	Minus      TokenType = 7
	Plus       TokenType = 8
	Semicolon  TokenType = 9
	Slash      TokenType = 10
	Star       TokenType = 11

	// one or two character tokens
	Bang         TokenType = 12
	BangEqual    TokenType = 13
	Equal        TokenType = 14
	EqualEqual   TokenType = 15
	Greater      TokenType = 16
	GreaterEqual TokenType = 17
	Less         TokenType = 18
	LessEqual    TokenType = 19

	// literals
	Identifier TokenType = 20
	String     TokenType = 21
	Number     TokenType = 22

	// keywords
	And    TokenType = 23
	Or     TokenType = 24
	If     TokenType = 25
	Else   TokenType = 26
	Class  TokenType = 27
	Super  TokenType = 28
	This   TokenType = 29
	True   TokenType = 30
	False  TokenType = 31
	Fun    TokenType = 32
	For    TokenType = 33
	While  TokenType = 34
	Nil    TokenType = 35
	Print  TokenType = 36
	Return TokenType = 37
	Var    TokenType = 38
)

type LiteralType int

const (
	LiteralTypeNone   LiteralType = 0
	LiteralTypeNumber LiteralType = 1
	LiteralTypeString LiteralType = 2
)

type LiteralValue struct {
	Type LiteralType `json:"type"`

	NumberValue float64 `json:"number"`
	StringValue string  `json:"string"`
}

func NewNumberLiteral(value float64) LiteralValue {
	return LiteralValue{
		Type:        LiteralTypeNumber,
		NumberValue: value,
	}
}

func NewStringLiteral(value string) LiteralValue {
	return LiteralValue{
		Type:        LiteralTypeNumber,
		StringValue: value,
	}
}

type Token struct {
	Type    TokenType    `json:"type"`
	Lexeme  string       `json:"lexeme,omitempty"`
	Literal LiteralValue `json:"literal,omitempty"`
	Line    uint         `json:"line"`
}
