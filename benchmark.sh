#! /bin/sh

GO=go
CARGO=cargo

JLOX=golox
CLOX=loxrs

cd $JLOX/
echo "Running fib.lox ..."
./$JLOX ../craftinginterpreters/test/benchmark/fib.lox
cd -
