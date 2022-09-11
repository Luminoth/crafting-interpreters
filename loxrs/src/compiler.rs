//! Lox compiler

use std::cell::RefCell;

use tracing::error;

use crate::chunk::*;
use crate::scanner::*;
use crate::vm::*;

/// Lox parser
#[derive(Debug)]
struct Parser<'a> {
    scanner: Scanner<'a>,

    current: RefCell<Token<'a>>,
    previous: RefCell<Token<'a>>,

    had_error: RefCell<bool>,
    panic_mode: RefCell<bool>,
}

impl<'a> Parser<'a> {
    fn new(scanner: Scanner<'a>) -> Self {
        Self {
            scanner,
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

    fn advance(&'a self) {
        *self.previous.borrow_mut() = *self.current.borrow();

        loop {
            *self.current.borrow_mut() = self.scanner.scan_token();
            if self.current.borrow().r#type != TokenType::Error {
                break;
            }

            self.error_at_current(self.current.borrow().lexeme.unwrap());
        }
    }

    fn consume(&'a self, r#type: TokenType, message: impl AsRef<str>) {
        if self.current.borrow().r#type == r#type {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

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
    let chunk = Chunk::new();

    let scanner = Scanner::new(&input);
    let parser = Parser::new(scanner);

    parser.advance();
    //parser.expression();
    parser.consume(TokenType::Eof, "Expect end of expression.");

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
