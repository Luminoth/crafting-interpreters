# Crafting Interpreters

## Lox

### Features

#### Builtins

* print

#### Types

* Dynamic typing
* Booleans
  * true
  * false
* Numbers
  * Integers
  * Floats
* Strings
* Nil

#### Expressions

* Arithmetic
  * Addition
  * Subtraction
  * Multiplication
  * Division
  * Negation
* Comparison and Equality
  * Less than
  * Less than equal
  * Greater than
  * Greater than equal
  * Equal
  * Not equal
* Logical Operators
  * Not
  * And
  * Or
* Control Flow
  * If
  * While
  * For
* Functions
* Closures
* Classes
  * Functions
  * Fields
  * Single Inheritence

#### Standard Library

* clock()

### Grammar

```
expression -> literal | unary | binary | grouping ;
literal -> NUMBER | STRING | "true" | "false" | "nil" ;
grouping -> "(" expression ")" ;
unary -> ( "-" | "!" ) expression ;
binary -> expression operator expression ;
operator -> "==" | "!=" | "<" | "<=" | ">" | ">=" | "+" | "-" | "*" | "/" ;
```
