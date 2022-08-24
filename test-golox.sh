#! /bin/sh

echo "Running generator ..."
./generate-ast.py generate

echo "Building golox ..."
cd golox/
go build

cd -

echo "Running tests ..."
cd craftinginterpreters
#dart tool/bin/test.dart jlox --interpreter ../golox/golox
#dart tool/bin/test.dart clox --interpreter ../loxrs/loxrs
dart tool/bin/test.dart chap12_classes --interpreter ../golox/golox
