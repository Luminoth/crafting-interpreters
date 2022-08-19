package main

import "testing"

func scan(source string) Scanner {
	hadError = false

	scanner := NewScanner(source, true)
	scanner.ScanTokens()

	return scanner
}

func simpleScanTest(t *testing.T, source string, expectedTokens []TokenType, expectedLines uint) Scanner {
	expectedTokens = append(expectedTokens, EOF)

	scanner := scan(source)
	if hadError {
		t.Fatal("Unexpected parse error")
	}

	tokenCount := len(scanner.Tokens)
	expectedCount := len(expectedTokens)

	if tokenCount != expectedCount {
		t.Fatalf("Expected %d tokens, got %d", expectedCount, tokenCount)
	}

	for idx, token := range scanner.Tokens {
		expectedToken := expectedTokens[idx]
		if token.Type != expectedToken {
			t.Fatalf("Expected %s, got %v", expectedToken, token)
		}
	}

	if scanner.Line != expectedLines {
		t.Fatalf("Expected line %d, got %d", expectedLines, scanner.Line)
	}

	return scanner
}

func simpleErrorScanTest(t *testing.T, source string) Scanner {
	scanner := scan(source)
	if !hadError {
		t.Fatal("Expected parse error")
	}

	return scanner
}

func TestSingleTokens(t *testing.T) {
	source := "( ){ } , . - + ; / * ? :"
	expectedTokens := []TokenType{
		LeftParen, RightParen,
		LeftBrace, RightBrace,
		Comma, Dot,
		Minus, Plus,
		Semicolon, Slash, Star,
		Question, Colon,
	}

	simpleScanTest(t, source, expectedTokens, 1)
}

func TestMultiTokens(t *testing.T) {
	source := "! != = == > >= < <="
	expectedTokens := []TokenType{
		Bang, BangEqual,
		Equal, EqualEqual,
		Greater, GreaterEqual,
		Less, LessEqual,
	}

	simpleScanTest(t, source, expectedTokens, 1)
}

func TestIdentifier(t *testing.T) {
	source := "test that thing"
	expectedTokens := []TokenType{
		Identifier, Identifier, Identifier,
	}

	simpleScanTest(t, source, expectedTokens, 1)
}

func TestKeyword(t *testing.T) {
	source := "and or if else class super this true false fun for while nil print return var"
	expectedTokens := []TokenType{
		And, Or,
		If, Else,
		Class, Super, This,
		True, False,
		Fun,
		For, While,
		Nil,
		Print,
		Return,
		Var,
	}

	simpleScanTest(t, source, expectedTokens, 1)
}

func TestString(t *testing.T) {
	source := `"this is a string" "this is another string" "and one more string"
	"this is a multiline string
	and it's going to cover
	a few lines"`
	expectedTokens := []TokenType{
		String, String, String,
		String,
	}

	simpleScanTest(t, source, expectedTokens, 4)

	source = "\"this is an unterminated string"

	simpleErrorScanTest(t, source)
}

func TestNumber(t *testing.T) {
	source := "123 123.456 0.12"
	expectedTokens := []TokenType{
		Number, Number, Number,
	}

	simpleScanTest(t, source, expectedTokens, 1)
}

func TestSingleComment(t *testing.T) {
	source := `// this is a full line comment
	test // this is a comment after a line`
	expectedTokens := []TokenType{
		Identifier,
	}

	simpleScanTest(t, source, expectedTokens, 2)
}

func TestMultiComment(t *testing.T) {
	source := `/* this is a multi line comment
	that has a few lines
	to cover */
	test /*
	this multiline comment has * a few extra
	things in it / that we want to make sure work
	*/`
	expectedTokens := []TokenType{
		Identifier,
	}

	simpleScanTest(t, source, expectedTokens, 7)

	source = "/* this is an unterminated comment *"

	simpleErrorScanTest(t, source)
}
