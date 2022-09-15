//! Lox compiler

use std::cell::RefCell;

use tracing::error;

use crate::chunk::*;
use crate::scanner::*;
use crate::value::*;
use crate::vm::*;

/// Precedence levels, lowest to highest
#[derive(
    Debug,
    Copy,
    Clone,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    strum_macros::Display,
    strum_macros::AsRefStr,
    strum_macros::FromRepr,
)]
enum Precedence {
    None,
    Assignment, // =
    Ternary,    // ?:
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

impl Precedence {
    #[inline]
    fn next(&self) -> Precedence {
        Self::from_repr(*self as usize + 1).unwrap()
    }
}

impl TokenType {
    /// Pratt Parser precedence rule
    fn precedence(&self) -> Precedence {
        match self {
            Self::BangEqual | Self::EqualEqual => Precedence::Equality,
            Self::Greater | Self::GreaterEqual | Self::Less | Self::LessEqual => {
                Precedence::Comparison
            }
            Self::Minus | Self::Plus => Precedence::Term,
            Self::Slash | Self::Star => Precedence::Factor,
            Self::Question | Self::Colon => Precedence::Ternary,
            _ => Precedence::None,
        }
    }
}

/// Lox parser
///
/// Sort of implements the Pratt Parser from the book but without building the table
#[derive(Debug)]
struct Parser<'a> {
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

    #[inline]
    fn had_error(&self) -> bool {
        *self.had_error.borrow()
    }

    #[inline]
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

    /// Pratt Parser prefix parsing rule
    fn prefix(&mut self, r#type: TokenType) -> bool {
        match r#type {
            TokenType::Nil | TokenType::False | TokenType::True => self.literal(),
            TokenType::LeftParen => self.grouping(),
            TokenType::Minus | TokenType::Bang => self.unary(),
            TokenType::Number => self.number(),
            _ => return false,
        }

        true
    }

    /// Pratt Praser infix parsing rule
    fn infix(&mut self, r#type: TokenType) -> bool {
        match r#type {
            TokenType::BangEqual
            | TokenType::EqualEqual
            | TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual
            | TokenType::Minus
            | TokenType::Plus
            | TokenType::Slash
            | TokenType::Star => self.binary(),
            TokenType::Question => self.ternary(),
            _ => return false,
        }

        true
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        // handle prefix expression to start
        let r#type = self.previous.borrow().r#type;
        if !self.prefix(r#type) {
            self.error("Expect expression.")
        }

        // handle infix expressions if there are any
        loop {
            let current = self.current.borrow().r#type.precedence();
            if precedence > current {
                break;
            }

            self.advance();

            let r#type = self.previous.borrow().r#type;
            self.infix(r#type);
        }
    }

    fn expression(&mut self) {
        // start with the lowest level precedence
        self.parse_precedence(Precedence::None.next());
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn binary(&mut self) {
        let operator = self.previous.borrow().r#type;
        self.parse_precedence(operator.precedence().next());

        match operator {
            TokenType::BangEqual => {
                #[cfg(feature = "extended_opcodes")]
                self.emit_instruction(OpCode::NotEqual);

                #[cfg(not(feature = "extended_opcodes"))]
                self.emit_instructions(&[OpCode::Equal, OpCode::Not]);
            }
            TokenType::EqualEqual => self.emit_instruction(OpCode::Equal),
            TokenType::Greater => self.emit_instruction(OpCode::Greater),
            TokenType::GreaterEqual => {
                #[cfg(feature = "extended_opcodes")]
                self.emit_instruction(OpCode::GreaterEqual);

                // a >= b == !(a < b)
                #[cfg(not(feature = "extended_opcodes"))]
                self.emit_instructions(&[OpCode::Less, OpCode::Not]);
            }
            TokenType::Less => self.emit_instruction(OpCode::Less),
            TokenType::LessEqual => {
                #[cfg(feature = "extended_opcodes")]
                self.emit_instruction(OpCode::LessEqual);

                // a <= b == !(a > b)
                #[cfg(not(feature = "extended_opcodes"))]
                self.emit_instructions(&[OpCode::Greater, OpCode::Not]);
            }
            TokenType::Plus => self.emit_instruction(OpCode::Add),
            TokenType::Minus => self.emit_instruction(OpCode::Subtract),
            TokenType::Star => self.emit_instruction(OpCode::Multiply),
            TokenType::Slash => self.emit_instruction(OpCode::Divide),
            _ => (),
        }
    }

    fn ternary(&mut self) {
        // TODO: emit the instructions associated with this

        // TODO: pretty sure this isn't right in terms of the precedences of this operator

        self.parse_precedence(TokenType::Question.precedence().next());

        self.consume(TokenType::Colon, "Expect ':' after expression.");

        self.parse_precedence(TokenType::Colon.precedence().next());
    }

    fn unary(&mut self) {
        let operator = self.previous.borrow().r#type;

        self.parse_precedence(Precedence::Unary);

        #[allow(clippy::single_match)]
        match operator {
            TokenType::Minus => self.emit_instruction(OpCode::Negate),
            TokenType::Bang => self.emit_instruction(OpCode::Not),
            _ => (),
        }
    }

    fn number(&mut self) {
        let value = self.previous.borrow().lexeme.unwrap().parse().unwrap();
        self.emit_constant(Value::Number(value));
    }

    fn literal(&mut self) {
        let token = self.previous.borrow().r#type;
        match token {
            TokenType::Nil => self.emit_instruction(OpCode::Nil),
            TokenType::False => self.emit_instruction(OpCode::False),
            TokenType::True => self.emit_instruction(OpCode::True),
            _ => (),
        }
    }

    fn emit_instruction(&mut self, instruction: OpCode) {
        self.chunk.write(instruction, self.previous.borrow().line);
    }

    fn emit_instructions(&mut self, instructions: impl AsRef<[OpCode]>) {
        for instruction in instructions.as_ref() {
            self.emit_instruction(*instruction);
        }
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let idx = self.chunk.add_constant(value);
        if idx > u8::MAX as usize {
            self.error("Too many constants in one chunk.");
            return 0;
        }
        idx as u8
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_instruction(OpCode::Constant(constant));
    }

    fn emit_return(&mut self) {
        self.emit_instruction(OpCode::Return);
    }

    fn end_compiler(&mut self) {
        self.emit_return();

        #[cfg(feature = "debug_code")]
        if !self.had_error() {
            self.chunk.disassemble("code");
        }
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
