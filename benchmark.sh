#! /bin/sh

GO=go
CARGO=cargo

JLOX=golox
CLOX=loxrs

echo "Running jlox benchmarks ..."

cd $JLOX/
echo "Running fib.lox ..."
./$JLOX ../craftinginterpreters/test/benchmark/fib.lox
cd -
