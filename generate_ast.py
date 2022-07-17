#!/usr/bin/env python3

import argparse
import io
import os
import sys
from typing import Dict, List


class LanguageConfig:
    def __init__(self, output_dir: str, output_file: str, format_cmd: str):
        self.output_dir = output_dir
        self.output_file = output_file
        self.format_cmd = format_cmd

        self.__output_file_path = os.path.join(self.output_dir, self.output_file)

    @property
    def output_file_path(self) -> str:
        return self.__output_file_path


class ExpressionDef:
    def __init__(self, name: str, fields: Dict[str, str]):
        self.name = name
        self.fields = fields


SUPPORTED_LANGUAGES = {
    'go': LanguageConfig(os.path.join('golox'), 'expression.go', 'gofmt -w'),
}

EXPRESSIONS = [
    ExpressionDef('Binary', {
        'left': 'Expression',
        'operator': 'Token',
        'right': 'Expression',
    }),
    ExpressionDef('Grouping', {
        'expression': 'Expression',
    }),
    ExpressionDef('Literal', {
        'value': 'Object',
    }),
    ExpressionDef('Unary', {
        'operator': 'Token',
        'right': 'Expression',
    }),
]


def generate_go_expression_visitor_acceptor(f: io.TextIOWrapper, expression: ExpressionDef):
    # type
    f.write('\n')
    f.write(f'type {expression.name}ExpressionAcceptor[T any] struct {{\n')
    f.write(f'Expression *{expression.name}Expression\n')
    f.write('}\n')

    # constructor
    f.write('\n')
    f.write(
        f'func New{expression.name}ExpressionAcceptor[T any](expression *{expression.name}Expression) *{expression.name}ExpressionAcceptor[T] {{\n')
    f.write(f'return &{expression.name}ExpressionAcceptor[T]{{\n')
    f.write(f'Expression: expression,\n')
    f.write('}\n')
    f.write('}\n')

    # acceptor
    f.write('\n')
    f.write(
        f'func(a *{expression.name}ExpressionAcceptor[T]) Accept(visitor ExpressionVisitor[T]) T {{\n')
    f.write(f'return visitor.Visit{expression.name}Expression(a.Expression)\n')
    f.write('}\n')


def generate_go_visitors(f: io.TextIOWrapper):
    # visitor interface
    f.write('\n')
    f.write('type ExpressionVisitor[T any] interface {\n')
    for expression in EXPRESSIONS:
        f.write(f'Visit{expression.name}Expression(expression *{expression.name}Expression) T\n')
    f.write('}\n')

    # Go doesn't support generics in method receivers
    # but we can use a facilitator to get around that
    # https://rakyll.org/generics-facilititators/
    f.write("""
type ExpressionVisitorFacilitator[T any] interface {
    Accept(visitor ExpressionVisitor[T]) T
}
""")

    # acceptors
    for expression in EXPRESSIONS:
        generate_go_expression_visitor_acceptor(f, expression)


def generate_go_expression(f: io.TextIOWrapper, expression: ExpressionDef):
    # expression type
    f.write('\n')
    f.write(f'type {expression.name}Expression struct {{\n')
    for name, type in expression.fields.items():
        # do some type overriding
        match type:
            case 'Object':
                type = 'interface{}'
            case 'Token':
                type = '*Token'
        f.write(f'{name.capitalize()} {type}\n')
    f.write('}\n')


def generate_go():
    file_path = SUPPORTED_LANGUAGES['go'].output_file_path
    print(f'Generating Go AST to "{file_path}" ...')

    with open(file_path, 'w', encoding='utf-8') as f:
        # header
        f.write('package main\n')

        # expression interface
        f.write("""
type Expression interface {
}
""")

        # expressions
        for expression in EXPRESSIONS:
            generate_go_expression(f, expression)

        # visitors
        generate_go_visitors(f)

    # format the file
    format_cmd = f'{SUPPORTED_LANGUAGES["go"].format_cmd} {file_path}'
    print(f'Formatting output "{format_cmd}" ...')
    os.system(format_cmd)


def generate(languages: List[str]):
    if 'go' in languages:
        generate_go()


def main(args: argparse.Namespace):
    match args.command:
        case 'generate':
            generate(args.languages)
        case _:
            print(f'Unsupported command: {args.command}')
            sys.exit(1)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='AST codegen')

    subparsers = parser.add_subparsers(help='Command help', dest='command')
    subparsers.required = True

    subparser = subparsers.add_parser('generate')
    subparser.add_argument('--languages', choices=SUPPORTED_LANGUAGES.keys(), default=SUPPORTED_LANGUAGES.keys(),
                           help='Which languages to generate for')

    args = parser.parse_args()

    main(args)
