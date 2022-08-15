package main

type Stack[T any] []T

func (s *Stack[T]) IsEmpty() bool {
	return len(*s) == 0
}

func (s *Stack[T]) Push(v T) {
	*s = append(*s, v)
}

func (s *Stack[T]) Pop() (T, bool) {
	if s.IsEmpty() {
		return *new(T), false
	}

	idx := len(*s) - 1
	v := (*s)[idx]
	*s = (*s)[:idx]
	return v, true

}

func (s *Stack[T]) Peek() (T, bool) {
	if s.IsEmpty() {
		return *new(T), false
	}

	idx := len(*s) - 1
	v := (*s)[idx]
	return v, true

}
