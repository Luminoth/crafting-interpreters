package main

import "testing"

func TestMin(t *testing.T) {
	if Min(1, 2) != 1 {
		t.Fatalf("Min(1, 2) != 1")
	}

	if Min(2, 1) != 1 {
		t.Fatal("Min(2, 1) != 1")
	}

	if Min(1.0, 2.0) != 1.0 {
		t.Fatal("Min(1.0, 2.0) != 1.0")
	}

	if Min(2.0, 1.0) != 1.0 {
		t.Fatal("Min(1.0, 2.0) != 1.0")
	}
}

func TestAlpha(t *testing.T) {
	for r := 'a'; r <= 'z'; r++ {
		if !IsAlpha(r) {
			t.Fatalf("IsAlpha('%c')", r)
		}
	}

	for r := 'A'; r <= 'Z'; r++ {
		if !IsAlpha(r) {
			t.Fatalf("IsAlpha('%c')", r)
		}
	}

	if !IsAlpha('_') {
		t.Fatal("IsAlpha('_')")
	}

	for r := '0'; r <= '9'; r++ {
		if IsAlpha(r) {
			t.Fatalf("IsAlpha('%c')", r)
		}
	}

	if IsAlpha('!') {
		t.Fatal("IsAlpha('!')")
	}
}

func TestAlphaNumeric(t *testing.T) {
	for r := 'a'; r <= 'z'; r++ {
		if !IsAlphaNumeric(r) {
			t.Fatalf("IsAlphaNumeric('%c')", r)
		}
	}

	for r := 'A'; r <= 'Z'; r++ {
		if !IsAlphaNumeric(r) {
			t.Fatalf("IsAlphaNumeric('%c')", r)
		}
	}

	if !IsAlphaNumeric('_') {
		t.Fatal("IsAlphaNumeric('_')")
	}

	for r := '0'; r <= '9'; r++ {
		if !IsAlphaNumeric(r) {
			t.Fatalf("IsAlphaNumeric('%c')", r)
		}
	}

	if IsAlphaNumeric('!') {
		t.Fatal("IsAlphaNumeric('!')")
	}
}
