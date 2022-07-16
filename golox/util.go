package main

import (
	"unicode"

	"golang.org/x/exp/constraints"
)

func Min[T constraints.Ordered](x, y T) T {
	if x < y {
		return x
	}
	return y
}

func IsAlpha(ch rune) bool {
	return unicode.IsLetter(ch) || ch == '_'
}

func IsAlphaNumeric(ch rune) bool {
	return unicode.IsLetter(ch) || ch == '_' || unicode.IsDigit(ch)
}
