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


def generate_go_expression(f: io.TextIOWrapper, expression: ExpressionDef):
    f.write('\n')
    f.write(f'type {expression.name}Expression struct {{\n')
    for name, type in expression.fields.items():
        if type == 'Object':
            type = 'interface{}'
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