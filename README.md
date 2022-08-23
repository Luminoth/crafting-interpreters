# Crafting Interpreters

* https://github.com/munificent/craftinginterpreters
* https://en.cppreference.com/w/c/language/operator_precedence
* https://github.com/fsacer/FailLang
  * Interesting expansion on Lox

## Lox Test Suite

* Checkout https://github.com/munificent/craftinginterpreters
* Run `make get` from inside `craftinginterpreters`
  * This requires Flutter to be installed
* Run the regression scripts from the root
  * eg. `./test-golox.sh`

## Lox

### Features

* Scope - A variable usage refers to the preceding declaration with the same name in the innermost scope that encloses the expression where the variable is used.

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

### Deviations

* Multi-line comments
* Comma expressions
* Ternary operator
* Extended string concatenation
* Divide by zero runtime error
* Prompt prints expression results
* break and continue

### TODO:

* Chapter 10
  * Anonymous functions (lambdas)
* Chapter 11
  * Unused variable warning
  * Environment optimization

### Grammar

```
program                 -> declaration* EOF ;
declaration             -> function_declaration
                        | variable_declaration
                        | statement ;
function_declaration    -> "fun" function ;
function                -> IDENTIFIER ( parameters? ) block ";"
parameters              -> IDENTIFIER ( "," IDENTIFIER )* ;
variable_declaration    -> "var" IDENTIFIER ( "=" expression )? ";" ;
statement               -> expression_statement
                        | for_statement
                        | if_statement
                        | print_statement
                        | return_statement
                        | while_statement
                        | break_statement
                        | continue_statement
                        | block ;
expression_statement    -> expression ";" ;
for_statement           -> "for" "(" ( variable_declaration | expression_statement | ";" )
                            expression? ";"
                            expression? ")" statement ;
if_statement            -> "if" "(" expression ")" statement
                            ( "else" statement )? ;
print_statement         -> "print" expression ";" ;
return_statement        -> "return" expression? ";" ;
while_statement         -> "while" "(" expression ")" statement ;
break_statement         -> "break" ";" ;
continue_statement      -> "continue" ";" ;
block                   -> "{" declaration* "}" ;
expression              -> assignment ( "," expression )* ;
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
                        | call ;
call                    -> primary ( "(" arguments? ")" )* ;
arguments               -> assignment ( "," assignment )* ;
primary                 -> NUMBER | STRING
                        | "true" | "false" | "nil"
                        | "(" expression ")"
                        | IDENTIFIER ;
```

### Operator Precedence

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
