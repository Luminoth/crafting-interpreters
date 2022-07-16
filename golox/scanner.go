package main

type Scanner struct {
}

func (s *Scanner) ScanTokens() []Token {
	return []Token{}
}

func NewScanner(source string) Scanner {
	return Scanner{}
}
