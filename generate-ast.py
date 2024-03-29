#!/usr/bin/env python3

import argparse
import io
import os
import sys
from abc import ABC, abstractmethod
from typing import Dict, List, Optional


class ASTDef:
    def __init__(self, name: str, fields: Dict[str, str], note: Optional[str] = None):
        self.name = name
        self.fields = fields
        self.note = note


EXPRESSIONS = [
    ASTDef('Assign', {
        'name': 'Token',
        'value': 'Expression',
    }),
    ASTDef('Binary', {
        'left': 'Expression',
        'operator': 'Token',
        'right': 'Expression',
    }),
    ASTDef('Call', {
        'callee': 'Expression',
        'paren': 'Token',
        'arguments': 'List[Expression]',
    }),
    ASTDef('Get', {
        'object': 'Expression',
        'name': 'Token',
    }),
    ASTDef('Grouping', {
        'expression': 'Expression',
    }),
    ASTDef('Literal', {
        'value': 'Object',
    }),
    ASTDef('Logical', {
        'left': 'Expression',
        'operator': 'Token',
        'right': 'Expression',
    }),
    ASTDef('Set', {
        'object': 'Expression',
        'name': 'Token',
        'value': "Expression",
    }),
    ASTDef('Super', {
        'keyword': 'Token',
        'method': 'Token',
    }),
    ASTDef('This', {
        'keyword': 'Token',
    }),
    ASTDef('Ternary', {
        'condition': 'Expression',
        'true': 'Expression',
        'false': 'Expression',
    }),
    ASTDef('Unary', {
        'operator': 'Token',
        'right': 'Expression',
    }),
    ASTDef('Variable', {
        'name': 'Token',
    }),
]

STATEMENTS = [
    ASTDef('Block', {
        'statements': 'List[Statement]',
    }),
    ASTDef('Break', {
        'keyword': 'Token',
    }),
    ASTDef('Class', {
        'name': 'Token',
        'superclass': 'VariableExpression',
        'methods': 'List[Function]',
    }),
    ASTDef('Continue', {
        'keyword': 'Token',
    }),
    ASTDef('Expression', {
        'expression': 'Expression',
    }),
    ASTDef('Function', {
        'name': 'Token',
        'params': 'List[Token]',
        'body': 'List[Statement]',
    }),
    ASTDef('If', {
        'condition': 'Expression',
        'then': 'Statement',
        'else': 'Statement',
    }),
    ASTDef('Print', {
        'expression': 'Expression',
    }),
    ASTDef('Return', {
        'keyword': 'Token',
        'value': 'Expression',
    }),
    ASTDef('Var', {
        'name': 'Token',
        'initializer': 'Expression',
    }),
    ASTDef('While', {
        'condition': 'Expression',
        'body': 'Statement',
    }, 'For statement desugars to a While statement'),
]


class Generator(ABC):
    def __init__(self, output_dir: str, extension: str, format_cmd: str):
        # TODO: instead of passing anything into this we can just use
        # some more abstract properties and methods for them

        self.__format_cmd = format_cmd

        self.__expression_output_file_path = os.path.join(output_dir, f'expression.{extension}')
        self.__statement_output_file_path = os.path.join(output_dir, f'statement.{extension}')

    @property
    @abstractmethod
    def language(self) -> str:
        pass

    @abstractmethod
    def _write_header(self, type: str, f: io.TextIOWrapper):
        pass

    @abstractmethod
    def _write_interface(self, type: str, f: io.TextIOWrapper):
        pass

    @abstractmethod
    def _generate_visitors(self, type: str, ast_defs: List[ASTDef], f: io.TextIOWrapper):
        pass

    @abstractmethod
    def _generate_definition(self, type: str, ast_def: ASTDef, f: io.TextIOWrapper):
        pass

    def __generate_definitions(self, type: str, file_path: str, ast_defs: List[ASTDef]):
        print(f'Generating {self.language} {type}s to "{file_path}" ...')

        with open(file_path, 'w', encoding='utf-8') as f:
            self._write_header(type, f)
            self._write_interface(type, f)

            for ast_def in ast_defs:
                self._generate_definition(type, ast_def, f)

            self._generate_visitors(type, ast_defs, f)

        # format the file
        format_cmd = f'{self.__format_cmd} {file_path}'
        print(f'Formatting output "{format_cmd}" ...')
        os.system(format_cmd)

    def __generate_expressions(self):
        self.__generate_definitions(
            'Expression', self.__expression_output_file_path, EXPRESSIONS)

    def __generate_statements(self):
        self.__generate_definitions(
            'Statement', self.__statement_output_file_path, STATEMENTS)

    def generate(self):
        self.__generate_expressions()
        self.__generate_statements()


class GoGenerator(Generator):
    def __init__(self):
        super().__init__(os.path.join('golox'), 'go', 'gofmt -w')

        self.__constraints = {
            'Expression': ['string', 'Value'],
        }

    @property
    def language(self) -> str:
        return "Go"

    def _write_header(self, type: str, f: io.TextIOWrapper):
        f.write("""/* This file is autogenerated, DO NOT MODIFY */
package main
""")

    def _write_interface(self, type: str, f: io.TextIOWrapper):
        f.write(f'\ntype {type} interface {{')
        if type in self.__constraints:
            for constraint in self.__constraints[type]:
                f.write(
                    f'Accept{constraint.capitalize()}(visitor {type}Visitor[{constraint}]) ({constraint}, error)\n')
        else:
            f.write(
                f'Accept(visitor {type}Visitor) (*Value, error)\n')
        f.write('}\n')

    def _generate_visitors(self, type: str, ast_defs: List[ASTDef], f: io.TextIOWrapper):
        # visitor type constraint
        if type in self.__constraints:
            f.write(f"""
type {type}VisitorConstraint interface {{
    {' | '.join(self.__constraints[type])}
}}
""")

        # visitor interface
        if type in self.__constraints:
            f.write(f'\ntype {type}Visitor[T {type}VisitorConstraint] interface {{\n')
            for ast_def in ast_defs:
                f.write(
                    f'Visit{ast_def.name}{type}({type.lower()} *{ast_def.name}{type}) (T, error)\n')
        else:
            f.write(f'\ntype {type}Visitor interface {{\n')
            for ast_def in ast_defs:
                f.write(
                    f'Visit{ast_def.name}{type}({type.lower()} *{ast_def.name}{type}) (*Value, error)\n')
        f.write('}\n')

    def _generate_definition(self, type: str, ast_def: ASTDef, f: io.TextIOWrapper):
        # type
        f.write('\n')
        if ast_def.note:
            f.write(f'// {ast_def.note}\n')
        f.write(f'type {ast_def.name}{type} struct {{\n')
        for field_name, field_type in ast_def.fields.items():
            # do some type overriding
            match field_type:
                case 'Object':
                    field_type = 'LiteralValue'
                case 'Token':
                    field_type = '*Token'
                case 'VariableExpression':
                    field_type = '*VariableExpression'
                case 'List[Expression]':
                    field_type = '[]Expression'
                case 'List[Statement]':
                    field_type = '[]Statement'
                case 'List[Function]':
                    field_type = '[]*FunctionStatement'
                case 'List[Token]':
                    field_type = '[]*Token'
            f.write(f'{field_name.capitalize()} {field_type}\n')
        f.write('}\n')

        # visitor interface
        if type in self.__constraints:
            for constraint in self.__constraints[type]:
                f.write(f"""
func (e *{ast_def.name}{type}) Accept{constraint.capitalize()}(visitor {type}Visitor[{constraint}]) ({constraint}, error) {{
    return visitor.Visit{ast_def.name}{type}(e)
}}
""")
        else:
            f.write(f"""
func (e *{ast_def.name}{type}) Accept(visitor {type}Visitor) (*Value, error) {{
    return visitor.Visit{ast_def.name}{type}(e)
}}
""")


GENERATORS = {
    'go': GoGenerator(),
}


def generate(languages: List[str]):
    for language in languages:
        if language in GENERATORS:
            GENERATORS[language].generate()


def main(args: argparse.Namespace):
    match args.command:
        case 'generate':
            generate(args.languages)
        case _:
            print(f'Unsupported command: {args.command}')
            sys.exit(1)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='AST codegen')

    subparsers = parser.add_subparsers(dest='command', help='sub-command help', required=True)

    subparser = subparsers.add_parser('generate')
    subparser.add_argument('--languages', nargs='+', choices=GENERATORS.keys(), default=GENERATORS.keys(),
                           help='Which languages to generate for')

    args = parser.parse_args()

    main(args)
