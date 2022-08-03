# Crafting Interpreters

* https://github.com/munificent/craftinginterpreters
* https://en.cppreference.com/w/c/language/operator_precedence
* https://github.com/fsacer/FailLang
  * Interesting expansion on Lox

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
program                 -> declaration* EOF ;
declaration             -> variable_declaration
                        | statement ;
variable_declaration    -> "var" IDENTIFIER ( "=" expression )? ";" ;
statement               -> expression_statement
                        | if_statement
                        | print_statement
                        | block ;
expression_statement    -> expression ";" ;
if_statement            -> "if" "(" expression ")" statement
                            ( "else" statement )? ;
print_statement         -> "print" expression ";" ;
block                   -> "{" declaration* "}" ;
expression              -> comma ;
comma                   -> assignment ( "," assignment )* ;
assignment              -> IDENTIFIER "=" assignment
                        | ternary ;
ternary                 -> logical_or ( "?" expression ":" ternary )? ;
logical_or              -> logical_and ( "or" logical_and )* ;
logical_and             -> equality ( "and" equality )* ;
equality                -> comparison ( ( "!=" | "==" ) comparison )* ;
comparison              -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term                    -> factor ( ( "-" | "+" ) factor )* ;
factor                  -> unary ( ( "/" | "*" ) unary )* ;
unary                   -> ( "!" | "-" ) unary
                        | primary ;
primary                 -> NUMBER | STRING
                        | "true" | "false" | "nil"
                        | "(" expression ")"
                        | IDENTIFIER ;
```

### Precedence

| Name        | Operators | Associates |
| ----------- | --------- |----------- |
| Unary       | ! -       | Right      |
| Factor      | / *       | Left       |
| Term        | - +       | Left       |
| Comparison  | > >= < <= | Left       |
| Equality    | == !=     | Left       |
| Logical And | and       | Left       |
| Logical Or  | or        | Left       |
| Ternary     | ?:        | Right      |
| Assignment  | =         | Right      |
| Comma       | ,         | Left       |
