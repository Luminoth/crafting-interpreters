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
    fn next(&self) -> Self {
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

const LOCALS_MAX: usize = std::u8::MAX as usize + 1;

/// Local variable state
#[derive(Debug, Default)]
struct Local<'a> {
    name: Token<'a>,
    depth: isize,
}

/// Lox compiler state
#[derive(Debug)]
struct Compiler<'a> {
    locals: [Local<'a>; LOCALS_MAX],
    local_count: usize,
    scope_depth: isize,
}

impl<'a> Default for Compiler<'a> {
    fn default() -> Self {
        let locals = [(); LOCALS_MAX].map(|_| Local::default());

        Self {
            locals,
            local_count: 0,
            scope_depth: 0,
        }
    }
}

impl<'a> Compiler<'a> {
    pub fn is_local_scope(&self) -> bool {
        self.scope_depth > 0
    }

    pub fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    pub fn end_scope(&mut self) -> usize {
        self.scope_depth -= 1;

        let mut count = 0;
        loop {
            if self.local_count == 0 || self.locals[self.local_count - 1].depth <= self.scope_depth
            {
                break;
            }

            count += 1;
            self.local_count -= 1;
        }

        count
    }

    pub fn is_local_declared(&self, name: &Token<'a>) -> bool {
        // current scope should be at the end of the set
        for idx in (0..self.local_count).rev() {
            let local = &self.locals[idx];

            // if we're outside the current scope, we're good
            if local.depth != -1 && local.depth < self.scope_depth {
                return false;
            }

            if name.lexeme == local.name.lexeme {
                return true;
            }
        }

        false
    }

    pub fn add_local(&mut self, name: Token<'a>) -> Result<(), &'static str> {
        if self.local_count >= LOCALS_MAX {
            return Err("Too many local variables in function");
        }

        let local = &mut self.locals[self.local_count];
        self.local_count += 1;

        local.name = name;
        local.depth = self.scope_depth;

        Ok(())
    }
}

/// Lox parser
///
/// Sort of implements the Pratt Parser from the book but without building the table
#[derive(Debug)]
struct Parser<'a> {
    compiler: Compiler<'a>,

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
            compiler: Compiler::default(),
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
            if !self.check(TokenType::Error) {
                break;
            }

            self.error_at_current(self.current.borrow().lexeme.unwrap());
        }
    }

    #[inline]
    fn check(&self, r#type: TokenType) -> bool {
        self.current.borrow().r#type == r#type
    }

    #[inline]
    fn check_previous(&self, r#type: TokenType) -> bool {
        self.previous.borrow().r#type == r#type
    }

    fn r#match(&self, r#type: TokenType) -> bool {
        if !self.check(r#type) {
            return false;
        }

        self.advance();
        true
    }

    fn consume(&self, r#type: TokenType, error_message: impl AsRef<str>) {
        if self.check(r#type) {
            self.advance();
            return;
        }

        self.error_at_current(error_message);
    }

    /// Pratt Parser prefix parsing rule
    fn prefix(&mut self, r#type: TokenType, can_assign: bool, vm: &VM) -> bool {
        match r#type {
            TokenType::Nil | TokenType::False | TokenType::True => self.literal(),
            TokenType::LeftParen => self.grouping(vm),
            TokenType::Minus | TokenType::Bang => self.unary(vm),
            TokenType::String => self.string(vm),
            TokenType::Number => self.number(),
            TokenType::Identifier => self.variable(can_assign, vm),
            _ => return false,
        }

        true
    }

    /// Pratt Praser infix parsing rule
    fn infix(&mut self, r#type: TokenType, _can_assign: bool, vm: &VM) -> bool {
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
            | TokenType::Star => self.binary(vm),
            TokenType::Question => self.ternary(vm),
            _ => return false,
        }

        true
    }

    fn parse_precedence(&mut self, precedence: Precedence, vm: &VM) {
        self.advance();

        // assignment is only allowed for lower precedences
        let can_assign = precedence <= Precedence::Assignment;

        // handle prefix expression to start
        let r#type = self.previous.borrow().r#type;
        if !self.prefix(r#type, can_assign, vm) {
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
            self.infix(r#type, can_assign, vm);
        }

        if can_assign && self.r#match(TokenType::Equal) {
            self.error("Invalid assignment target.");
        }
    }

    fn begin_scope(&mut self) {
        self.compiler.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.compiler.scope_depth -= 1;
    }

    fn identifier_constant(&mut self, name: impl AsRef<str>, vm: &VM) -> u8 {
        self.make_constant(Object::from_str(name.as_ref(), vm).into())
    }

    fn parse_variable(&mut self, vm: &VM, error_message: impl AsRef<str>) -> u8 {
        self.consume(TokenType::Identifier, error_message);

        self.declare_variable();

        // local variables don't go in the constants table
        if self.compiler.is_local_scope() {
            return 0;
        }

        let name = self.previous.borrow().lexeme.unwrap();
        self.identifier_constant(name, vm)
    }

    fn declare_variable(&mut self) {
        // global variables go in the constants table
        if !self.compiler.is_local_scope() {
            return;
        }

        let name = self.previous.borrow();

        // check for redeclarations
        if self.compiler.is_local_declared(&name) {
            self.error("Already a variable with this name in this scope.");
        }

        if let Err(err) = self.compiler.add_local(*name) {
            self.error(err);
        }
    }

    fn define_variable(&mut self, idx: u8) {
        // local variables are on the stack
        // rather than the globals table
        if self.compiler.is_local_scope() {
            return;
        }

        self.emit_instruction(OpCode::DefineGlobal(idx));
    }

    fn declaration(&mut self, vm: &VM) {
        // declaration -> variable_declaration | statement
        if self.r#match(TokenType::Var) {
            self.variable_declaration(vm);
        } else {
            self.statement(vm);
        }

        // error recovery
        if self.is_panic_mode() {
            self.synchronize();
        }
    }

    fn variable_declaration(&mut self, vm: &VM) {
        // variable_declaration -> "var" IDENTIFIER ( "=" expression )? ";"

        let global = self.parse_variable(vm, "Expect variable name.");

        // initializer
        if self.r#match(TokenType::Equal) {
            self.expression(vm);
        } else {
            self.emit_instruction(OpCode::Nil);
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );

        self.define_variable(global);
    }

    fn statement(&mut self, vm: &VM) {
        // statement -> expression_statement | print_statement | block

        // TODO: #[cfg(not(feature = "native_print"))]
        if self.r#match(TokenType::Print) {
            self.print_statement(vm);
            return;
        } else if self.r#match(TokenType::LeftBrace) {
            // blocks create new scopes
            self.compiler.begin_scope();
            self.block_statement(vm);

            let local_count = self.compiler.end_scope();
            for _ in 0..local_count {
                self.emit_instruction(OpCode::Pop);
            }

            return;
        }

        self.expression_statement(vm);
    }

    // TODO: #[cfg(not(feature = "native_print"))]
    fn print_statement(&mut self, vm: &VM) {
        // print_statement -> "print" expression ";"
        self.expression(vm);
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_instruction(OpCode::Print)
    }

    fn expression_statement(&mut self, vm: &VM) {
        // expression_statement -> expression ";"
        self.expression(vm);
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_instruction(OpCode::Pop);
    }

    fn block_statement(&mut self, vm: &VM) {
        // block -> "{" declaration* "}"
        loop {
            if self.check(TokenType::RightBrace) || self.check(TokenType::Eof) {
                break;
            }

            self.declaration(vm);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn expression(&mut self, vm: &VM) {
        // expression -> assignment ( "," expression )*
        // start with the lowest level precedence
        self.parse_precedence(Precedence::None.next(), vm);
    }

    fn grouping(&mut self, vm: &VM) {
        self.expression(vm);

        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn binary(&mut self, vm: &VM) {
        let operator = self.previous.borrow().r#type;
        self.parse_precedence(operator.precedence().next(), vm);

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

    fn ternary(&mut self, vm: &VM) {
        // ternary -> logical_or ( "?" expression ":" ternary )?

        // TODO: emit the instructions associated with this

        // TODO: pretty sure this isn't right in terms of the precedences of this operator

        self.parse_precedence(TokenType::Question.precedence().next(), vm);

        self.consume(TokenType::Colon, "Expect ':' after expression.");

        self.parse_precedence(TokenType::Colon.precedence().next(), vm);
    }

    fn unary(&mut self, vm: &VM) {
        let operator = self.previous.borrow().r#type;

        self.parse_precedence(Precedence::Unary, vm);

        #[allow(clippy::single_match)]
        match operator {
            TokenType::Minus => self.emit_instruction(OpCode::Negate),
            TokenType::Bang => self.emit_instruction(OpCode::Not),
            _ => (),
        }
    }

    fn string(&mut self, vm: &VM) {
        let value = self.previous.borrow().lexeme.unwrap();

        // string lexemes include the quotes, so we need to cut them off
        let value = &value[1..value.len() - 1];

        self.emit_constant(Object::from_str(value, vm).into());
    }

    fn number(&mut self) {
        let value = self
            .previous
            .borrow()
            .lexeme
            .unwrap()
            .parse::<f64>()
            .unwrap();
        self.emit_constant(value.into());
    }

    fn named_variable(&mut self, name: impl AsRef<str>, can_assign: bool, vm: &VM) {
        // TODO: this adds the constant to the chunk
        // even if it already exists, that's not great
        let idx = self.identifier_constant(name, vm);

        if can_assign && self.r#match(TokenType::Equal) {
            self.expression(vm);
            self.emit_instruction(OpCode::SetGlobal(idx));
        } else {
            self.emit_instruction(OpCode::GetGlobal(idx));
        }
    }

    fn variable(&mut self, can_assign: bool, vm: &VM) {
        let name = self.previous.borrow().lexeme.unwrap();
        self.named_variable(name, can_assign, vm);
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

    fn synchronize(&self) {
        *self.panic_mode.borrow_mut() = false;

        loop {
            if self.check(TokenType::Eof) {
                break;
            }

            // synchronize on statement end (semicolon)
            if self.check_previous(TokenType::Semicolon) {
                return;
            }

            // synchronize on statement begin
            let current = self.current.borrow().r#type;
            match current {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => (),
            }

            self.advance();
        }
    }
}

/// Compiles lox source
pub fn compile(input: impl AsRef<str>, vm: &VM) -> Result<Chunk, InterpretError> {
    let mut chunk = Chunk::new();

    let scanner = Scanner::new(input.as_ref());
    let mut parser = Parser::new(scanner, &mut chunk);

    // prime the parser
    parser.advance();

    // program -> declaration* EOF
    loop {
        if parser.r#match(TokenType::Eof) {
            break;
        }

        parser.declaration(vm);
    }

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
