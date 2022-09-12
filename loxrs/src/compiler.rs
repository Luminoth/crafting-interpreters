//! Lox compiler

use std::cell::RefCell;

use tracing::error;

use crate::chunk::*;
use crate::scanner::*;
use crate::value::*;
use crate::vm::*;

/// Precedence levels, lowest to highest
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // ==, !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

/// Lox parser
#[derive(Debug)]
pub struct Parser<'a> {
    scanner: Scanner<'a>,
    chunk: &'a mut Chunk,

    current: RefCell<Token<'a>>,
    previous: RefCell<Token<'a>>,

    had_error: RefCell<bool>,
    panic_mode: RefCell<bool>,
}

impl<'a> Parser<'a> {
    fn new(scanner: Scanner<'a>, chunk: &'a mut Chunk) -> Self {
        Self {
            scanner,
            chunk,
            current: RefCell::new(Token::default()),
            previous: RefCell::new(Token::default()),
            had_error: RefCell::new(false),
            panic_mode: RefCell::new(false),
        }
    }

    fn had_error(&self) -> bool {
        *self.had_error.borrow()
    }

    fn is_panic_mode(&self) -> bool {
        *self.panic_mode.borrow()
    }

    fn advance(&self) {
        *self.previous.borrow_mut() = *self.current.borrow();

        loop {
            // consume tokens until we hit one that is not an error
            *self.current.borrow_mut() = self.scanner.scan_token();
            if self.current.borrow().r#type != TokenType::Error {
                break;
            }

            self.error_at_current(self.current.borrow().lexeme.unwrap());
        }
    }

    fn consume(&self, r#type: TokenType, message: impl AsRef<str>) {
        if self.current.borrow().r#type == r#type {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    //#region should this go in a top-level compiler type?

    fn expression(&self) {
        // start with the lowest level precedence
        self.parse_precedence(Precedence::Assignment);
    }

    fn grouping(&'a self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn parse_precedence(&self, _precedence: Precedence) {
        // TODO: what goes here?
    }

    fn unary(&mut self) {
        let operator = self.previous.borrow().r#type;

        self.parse_precedence(Precedence::Unary);

        #[allow(clippy::single_match)]
        match operator {
            TokenType::Minus => self.emit_instruction(OpCode::Negate),
            _ => (),
        }
    }

    fn emit_instruction(&mut self, instruction: OpCode) {
        self.chunk.write(instruction, self.previous.borrow().line);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let idx = self.chunk.add_constant(value);
        if idx > u8::MAX as usize {
            self.error("Too many constants in one chunk.");
            return 0;
        }
        idx as u8
    }

    /// Emits a constant to the bytecode chunk
    pub fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_instruction(OpCode::Constant(constant));
    }

    fn emit_return(&mut self) {
        self.emit_instruction(OpCode::Return);
    }

    fn end_compiler(&mut self) {
        self.emit_return()
    }

    //#endregion

    fn error_at_current(&self, message: impl AsRef<str>) {
        self.error_at(&self.current.borrow(), message);
    }

    fn error(&self, message: impl AsRef<str>) {
        self.error_at(&self.previous.borrow(), message);
    }

    fn error_at(&self, token: &Token, message: impl AsRef<str>) {
        // only print the first error
        if self.is_panic_mode() {
            return;
        }
        *self.panic_mode.borrow_mut() = true;

        error!(
            "[line {}] Error{}: {}",
            token.line,
            if token.r#type == TokenType::Eof {
                " at end".to_owned()
            } else if token.r#type == TokenType::Error {
                // nothing
                "".to_owned()
            } else {
                format!(" at '{}'", token.lexeme.unwrap())
            },
            message.as_ref()
        );

        *self.had_error.borrow_mut() = true;
    }
}

/// Compiles lox source
pub fn compile(input: String) -> Result<Chunk, InterpretError> {
    let mut chunk = Chunk::new();

    let scanner = Scanner::new(&input);
    let mut parser = Parser::new(scanner, &mut chunk);

    parser.advance();
    parser.expression();
    parser.consume(TokenType::Eof, "Expect end of expression.");
    parser.end_compiler();

    if parser.had_error() {
        Err(InterpretError::Compile)
    } else {
        Ok(chunk)
    }
}

#[cfg(test)]
mod tests {
    // TODO:
}
