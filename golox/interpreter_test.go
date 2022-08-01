package main

/*func TestBinaryExpression(t *testing.T) {
	// -123 * (45.67)
	expectedResult := "-5617.41"

	expression := &BinaryExpression{
		Left: &UnaryExpression{
			Operator: &Token{
				Type:   Minus,
				Lexeme: "-",
			},
			Right: &LiteralExpression{
				Value: NewNumberLiteral(123),
			},
		},
		Operator: &Token{
			Type:   Star,
			Lexeme: "*",
		},
		Right: &GroupingExpression{
			Expression: &LiteralExpression{
				Value: NewNumberLiteral(45.67),
			},
		},
	}

	interpreter := NewInterpreter()
	_, result := interpreter.Interpret(expression)

	if result != expectedResult {
		t.Fatalf("Interpret failed - expected %s, got %s", expectedResult, result)
	}
}*/
