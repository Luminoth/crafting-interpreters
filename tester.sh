#! /bin/sh

GO=go
CARGO=cargo

JLOX=golox
CLOX=loxrs

echo "Running generator ..."
./generate-ast.py generate

echo "Running jlox tests ..."

echo "Building $JLOX ..."
cd $JLOX/
$GO build
cd -

cd craftinginterpreters
dart tool/bin/test.dart jlox --interpreter ../$JLOX/$JLOX
cd -

echo

echo "Running clox tests ..."

echo "Building $CLOX ..."
cd $CLOX/
$CARGO build
cp target/debug/$CLOX .
cd -

cd craftinginterpreters
#dart tool/bin/test.dart clox --interpreter ../$CLOX/$CLOX
dart tool/bin/test.dart chap19_strings --interpreter ../$CLOX/$CLOX
cd -
