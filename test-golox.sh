#! /bin/sh

echo "Running generator ..."
./generate-ast.py generate

echo "Building golox ..."
cd golox/
go build

cd -

echo "Running tests ..."
cd craftinginterpreters
dart tool/bin/test.dart jlox --interpreter ../golox/golox
