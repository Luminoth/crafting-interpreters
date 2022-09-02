#! /bin/sh

GO=go
CARGO=cargo

JLOX=golox
CLOX=loxrs

echo "Running generator ..."
./generate-ast.py generate

echo "Running jlox benchmarks ..."

echo "Building $JLOX ..."
cd $JLOX/
$GO build
cd -

cd craftinginterpreters
echo "Running fib.lox ..."
../$JLOX/$JLOX test/benchmark/fib.lox
cd -
