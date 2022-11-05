//! Lox parser / compiler

#![allow(dead_code)]

// TODO: I would definitely prefer a better split between the parser and the compiler here

use std::cell::RefCell;
use std::rc::Rc;

use tracing::error;

use crate::chunk::*;
use crate::object::*;
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
    #[cfg(feature = "ternary")]
    Ternary, // ?:
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
    #[inline]
    fn precedence(&self) -> Precedence {
        match self {
            #[cfg(feature = "ternary")]
            Self::Question | Self::Colon => Precedence::Ternary,
            Self::And => Precedence::And,
            Self::Or => Precedence::Or,
            Self::BangEqual | Self::EqualEqual => Precedence::Equality,
            Self::Greater | Self::GreaterEqual | Self::Less | Self::LessEqual => {
                Precedence::Comparison
            }
            Self::Minus | Self::Plus => Precedence::Term,
            Self::Slash | Self::Star => Precedence::Factor,
            _ => Precedence::None,
        }
    }
}

const LOCALS_MAX: usize = std::u8::MAX as usize + 1;

/// Local variable state
#[derive(Debug, Default)]
struct Local<'a> {
    name: Token<'a>,
    depth: Option<usize>,
}

#[cfg(feature = "dynamic_locals")]
type Locals<'a> = Vec<Local<'a>>;
#[cfg(not(feature = "dynamic_locals"))]
type Locals<'a> = [Local<'a>; LOCALS_MAX];

/// Function types
#[derive(Debug, PartialEq, Eq)]
enum FunctionType {
    Function,
    Script,
}

/// Lox compiler state
#[derive(Debug)]
struct Compiler<'a> {
    /// The current function being compiled
    ///
    /// This must be a Function Object
    function: Rc<Object>,

    /// The type of the current function being compiled
    function_type: FunctionType,

    /// The current scope depth
    scope_depth: usize,

    /// Local variable storage
    locals: Locals<'a>,

    /// The local variable count
    #[cfg(not(feature = "dynamic_locals"))]
    local_count: usize,
}

impl<'a> Compiler<'a> {
    fn new(function: Rc<Object>, function_type: FunctionType) -> Self {
        #[cfg(feature = "dynamic_locals")]
        let locals = Locals::with_capacity(LOCALS_MAX);
        #[cfg(not(feature = "dynamic_locals"))]
        let locals = [(); LOCALS_MAX].map(|_| Local::default());

        let mut this = Self {
            function,
            function_type,

            scope_depth: 0,
            locals,

            #[cfg(not(feature = "dynamic_locals"))]
            local_count: 0,
        };

        // local 0 is reserved for the VM
        this.push_local(Token::default());
        this.locals[0].depth = Some(0);

        this
    }

    fn new_function(name: impl AsRef<str>, vm: &VM) -> Self {
        let function = Object::function(name, vm);
        Self::new(function, FunctionType::Function)
    }

    fn new_script(vm: &VM) -> Self {
        let function = Object::script(vm);
        Self::new(function, FunctionType::Script)
    }

    /// Is this compiler currently in a local scope?
    #[inline]
    pub fn is_local_scope(&self) -> bool {
        self.scope_depth > 0
    }

    /// Begin a new local scope
    #[inline]
    pub fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    /// End the last local scope
    ///
    /// This returns the number of local values that should be pop'd from the stack
    pub fn end_scope(&mut self) -> usize {
        self.scope_depth -= 1;

        // purge the scope's locals
        let mut count = 0;
        loop {
            if self.local_count() == 0
                || self.locals[self.local_count() - 1].depth.unwrap() <= self.scope_depth
            {
                break;
            }

            count += 1;

            self.pop_local();
        }

        count
    }

    #[inline]
    fn local_count(&self) -> usize {
        #[cfg(feature = "dynamic_locals")]
        {
            self.locals.len()
        }

        #[cfg(not(feature = "dynamic_locals"))]
        {
            self.local_count
        }
    }

    fn push_local(&mut self, name: Token<'a>) {
        #[cfg(feature = "dynamic_locals")]
        {
            self.locals.push(Local {
                name,
                ..Default::default()
            });
        }

        #[cfg(not(feature = "dynamic_locals"))]
        {
            let local = &mut self.locals[self.local_count()];
            local.name = name;
            local.depth = None;

            self.local_count += 1;
        }
    }

    fn pop_local(&mut self) {
        #[cfg(feature = "dynamic_locals")]
        {
            self.locals.pop();
        }

        #[cfg(not(feature = "dynamic_locals"))]
        {
            self.local_count -= 1;
        }
    }

    /// Is the given local name currently declared?
    pub fn is_local_declared(&self, name: impl AsRef<str>) -> bool {
        // current scope is at the end of the set
        for idx in (0..self.local_count()).rev() {
            let local = &self.locals[idx];

            // if we're outside the current scope, we're good
            if let Some(depth) = local.depth {
                if depth < self.scope_depth {
                    return false;
                }
            }

            if name.as_ref() == local.name.lexeme.unwrap() {
                return true;
            }
        }

        false
    }

    /// Add a new local to the current scope
    pub fn add_local(&mut self, name: Token<'a>) -> Result<(), &'static str> {
        #[cfg(not(feature = "dynamic_locals"))]
        if self.local_count() >= LOCALS_MAX {
            return Err("Too many local variables in function");
        }

        self.push_local(name);

        Ok(())
    }

    /// Mark the last local as initialized
    pub fn mark_initialized(&mut self) {
        let idx = self.local_count() - 1;
        self.locals[idx].depth = Some(self.scope_depth);
    }

    /// Resolve the stack index of the given local
    pub fn resolve_local(&self, name: impl AsRef<str>) -> Result<Option<u8>, (u8, &'static str)> {
        // current scope is at the end of the set
        for idx in (0..self.local_count()).rev() {
            let local = &self.locals[idx];
            if name.as_ref() == local.name.lexeme.unwrap() {
                if local.depth.is_none() {
                    return Err((
                        idx as u8,
                        "Can't read local variable in its own initializer.",
                    ));
                }
                return Ok(Some(idx as u8));
            }
        }

        Ok(None)
    }
}

/// Lox parser
///
/// Sort of implements the Pratt Parser from the book but without building the table
#[derive(Debug)]
struct Parser<'a> {
    compiler: Compiler<'a>,

    scanner: Scanner<'a>,

    current: RefCell<Token<'a>>,
    previous: RefCell<Token<'a>>,

    had_error: RefCell<bool>,
    panic_mode: RefCell<bool>,
}

impl<'a> Parser<'a> {
    fn new(scanner: Scanner<'a>, vm: &VM) -> Self {
        Self {
            compiler: Compiler::new_script(vm),
            scanner,
            current: RefCell::new(Token::default()),
            previous: RefCell::new(Token::default()),
            had_error: RefCell::new(false),
            panic_mode: RefCell::new(false),
        }
    }

    fn get_name(&self) -> &str {
        match self.compiler.function.as_ref() {
            Object::Function(f) => f.get_name(),
            _ => unreachable!(),
        }
    }

    fn get_chunk_size(&self) -> usize {
        match self.compiler.function.as_ref() {
            Object::Function(f) => f.get_chunk().borrow().size(),
            _ => unreachable!(),
        }
    }

    #[cfg(any(feature = "debug_code", feature = "debug_trace"))]
    fn disassemble_chunk(&self, name: impl AsRef<str>) {
        match self.compiler.function.as_ref() {
            Object::Function(f) => f.get_chunk().borrow().disassemble(name),
            _ => unreachable!(),
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

    /// Advances the parser until it hits a non-error token
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

    /// Returns true if the current token is of the given type
    ///
    /// This does not consume the token
    #[inline]
    fn check(&self, r#type: TokenType) -> bool {
        self.current.borrow().r#type == r#type
    }

    /// Returns true if the previous token was of the given type
    #[inline]
    fn check_previous(&self, r#type: TokenType) -> bool {
        self.previous.borrow().r#type == r#type
    }

    /// Returns true if the current token is of the given type
    ///
    /// This consumes the token if it is of the given type
    fn r#match(&self, r#type: TokenType) -> bool {
        if !self.check(r#type) {
            return false;
        }

        self.advance();
        true
    }

    /// Consume the current token if it is of the given type
    ///
    /// Produce an error if the token is not of the given type
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
    fn infix(&mut self, r#type: TokenType, can_assign: bool, vm: &VM) -> bool {
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
            #[cfg(feature = "ternary")]
            TokenType::Question => self.ternary(vm),
            TokenType::And => self.and(can_assign, vm),
            TokenType::Or => self.or(can_assign, vm),
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
        self.compiler.begin_scope();
    }

    fn end_scope(&mut self) {
        let local_count = self.compiler.end_scope();
        for _ in 0..local_count {
            self.emit_instruction(OpCode::Pop);
        }
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
        if self.compiler.is_local_declared(name.lexeme.unwrap()) {
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
            self.compiler.mark_initialized();
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
        // statement -> expression_statement | if_statement | print_statement | block

        // TODO: #[cfg(not(feature = "native_print"))]
        if self.r#match(TokenType::Print) {
            self.print_statement(vm);
            return;
        }

        if self.r#match(TokenType::If) {
            self.if_statement(vm);
            return;
        }

        #[cfg(feature = "switch")]
        if self.r#match(TokenType::Switch) {
            self.switch_statement(vm);
            return;
        }

        if self.r#match(TokenType::While) {
            self.while_statement(vm);
            return;
        }

        if self.r#match(TokenType::For) {
            self.for_statement(vm);
            return;
        }

        if self.r#match(TokenType::LeftBrace) {
            // blocks create new scopes
            self.begin_scope();
            self.block_statement(vm);
            self.end_scope();
            return;
        }

        self.expression_statement(vm);
    }

    // TODO: #[cfg(not(feature = "native_print"))]
    fn print_statement(&mut self, vm: &VM) {
        // print_statement -> "print" expression ";"

        self.expression(vm);
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_instruction(OpCode::Print);
    }

    fn if_statement(&mut self, vm: &VM) {
        // if_statement -> "if" "(" expression ")" statement ( "else" statement )?

        // condition
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression(vm);
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        // then
        let then_idx = self.emit_instruction(OpCode::JumpIfFalse(0));
        self.emit_instruction(OpCode::Pop);

        self.statement(vm);

        // else
        let else_idx = self.emit_instruction(OpCode::Jump(0));

        self.patch_jump(then_idx);
        self.emit_instruction(OpCode::Pop);

        if self.r#match(TokenType::Else) {
            self.statement(vm);
        }
        self.patch_jump(else_idx);
    }

    #[cfg(feature = "switch")]
    fn switch_statement(&mut self, vm: &VM) {
        // switch_statement -> "switch" "(" expression ")" "{" switch_case* default_case? "}"

        // condition
        self.consume(TokenType::LeftParen, "Expect '(' after 'switch'.");
        self.expression(vm);
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        self.consume(TokenType::LeftBrace, "Expect '{' after condition.");

        loop {
            if self.r#match(TokenType::RightBrace) || self.check(TokenType::Eof) {
                break;
            }

            // TODO: handle the jumps required for this

            if self.r#match(TokenType::Case) {
                self.switch_case(vm);
            } else if self.r#match(TokenType::Default) {
                self.switch_default(vm);
            }
        }
    }

    #[cfg(feature = "switch")]
    fn switch_case(&mut self, vm: &VM) {
        // switch_case -> "case" expression ":" statement* ;

        // TODO: handle the jumps required for this

        self.expression(vm);

        self.consume(TokenType::Colon, "Expect ':' after expression.");

        loop {
            if self.check(TokenType::RightBrace)
                || self.check(TokenType::Case)
                || self.check(TokenType::Default)
                || self.check(TokenType::Eof)
            {
                break;
            }

            self.statement(vm);
        }
    }

    #[cfg(feature = "switch")]
    fn switch_default(&mut self, vm: &VM) {
        // default_case -> "default" ":" statement*

        // TODO: handle the jumps required for this

        self.consume(TokenType::Colon, "Expect ':' after expression.");

        loop {
            if self.check(TokenType::RightBrace)
                || self.check(TokenType::Case)
                || self.check(TokenType::Default)
                || self.check(TokenType::Eof)
            {
                break;
            }

            self.statement(vm);
        }
    }

    fn while_statement(&mut self, vm: &VM) {
        // while_statement -> "while" "(" expression ")" statement

        let loop_start = self.get_chunk_size();

        // condition
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression(vm);
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        // exit
        let exit_idx = self.emit_instruction(OpCode::JumpIfFalse(0));
        self.emit_instruction(OpCode::Pop);

        self.statement(vm);
        self.emit_loop(loop_start);

        self.patch_jump(exit_idx);
        self.emit_instruction(OpCode::Pop);
    }

    fn for_statement(&mut self, vm: &VM) {
        // for_statement -> "for" "(" ( variable_declaration | expression_statement | ";" ) expression? ";" expression? ")" statement

        self.begin_scope();

        // initializer
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");
        if self.r#match(TokenType::Semicolon) {
            // no initializer
        } else if self.r#match(TokenType::Var) {
            self.variable_declaration(vm);
        } else {
            self.expression_statement(vm);
        }

        let mut loop_start = self.get_chunk_size();

        // condition
        let mut exit_idx = None;
        if !self.r#match(TokenType::Semicolon) {
            self.expression(vm);
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");

            // exit
            exit_idx = Some(self.emit_instruction(OpCode::JumpIfFalse(0)));
            self.emit_instruction(OpCode::Pop);
        }

        // increment
        if !self.r#match(TokenType::RightParen) {
            // first pass jump to the body (no increment)
            let body_idx = self.emit_instruction(OpCode::Jump(0));

            let increment_start = self.get_chunk_size();
            self.expression(vm);
            self.emit_instruction(OpCode::Pop);

            self.consume(TokenType::RightParen, "Expect ')' after for clauses.");

            // loop start after the first pass is the increment
            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_idx);
        }

        self.statement(vm);
        self.emit_loop(loop_start);

        if let Some(exit_idx) = exit_idx {
            self.patch_jump(exit_idx);
            self.emit_instruction(OpCode::Pop);
        }

        self.end_scope();
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
                self.emit_instructions([OpCode::Equal, OpCode::Not]);
            }
            TokenType::EqualEqual => {
                self.emit_instruction(OpCode::Equal);
            }
            TokenType::Greater => {
                self.emit_instruction(OpCode::Greater);
            }
            TokenType::GreaterEqual => {
                #[cfg(feature = "extended_opcodes")]
                self.emit_instruction(OpCode::GreaterEqual);

                // a >= b == !(a < b)
                #[cfg(not(feature = "extended_opcodes"))]
                self.emit_instructions([OpCode::Less, OpCode::Not]);
            }
            TokenType::Less => {
                self.emit_instruction(OpCode::Less);
            }
            TokenType::LessEqual => {
                #[cfg(feature = "extended_opcodes")]
                self.emit_instruction(OpCode::LessEqual);

                // a <= b == !(a > b)
                #[cfg(not(feature = "extended_opcodes"))]
                self.emit_instructions([OpCode::Greater, OpCode::Not]);
            }
            TokenType::Plus => {
                self.emit_instruction(OpCode::Add);
            }
            TokenType::Minus => {
                self.emit_instruction(OpCode::Subtract);
            }
            TokenType::Star => {
                self.emit_instruction(OpCode::Multiply);
            }
            TokenType::Slash => {
                self.emit_instruction(OpCode::Divide);
            }
            _ => unreachable!(),
        }
    }

    #[cfg(feature = "ternary")]
    fn ternary(&mut self, vm: &VM) {
        // ternary -> logical_or ( "?" expression ":" ternary )?

        // TODO: should this use parse_precedence()?
        // I honestly have no idea what to do with this thing lol

        // condition
        // TODO: self.logical_or(vm);

        // left side has already been compiled
        // so all we have to do is jump if the result was false
        // otherwise finish parsing

        if !self.r#match(TokenType::Question) {
            return;
        }

        // then
        let then_idx = self.emit_instruction(OpCode::JumpIfFalse(0));
        self.emit_instruction(OpCode::Pop);

        self.expression(vm);

        self.consume(TokenType::Colon, "Expect ':' after expression.");

        // else
        let else_idx = self.emit_instruction(OpCode::Jump(0));

        self.patch_jump(then_idx);
        self.emit_instruction(OpCode::Pop);

        self.ternary(vm);

        self.patch_jump(else_idx);
    }

    fn unary(&mut self, vm: &VM) {
        let operator = self.previous.borrow().r#type;

        self.parse_precedence(Precedence::Unary, vm);

        #[allow(clippy::single_match)]
        match operator {
            TokenType::Minus => {
                self.emit_instruction(OpCode::Negate);
            }
            TokenType::Bang => {
                self.emit_instruction(OpCode::Not);
            }
            _ => unreachable!(),
        }
    }

    fn and(&mut self, _can_assign: bool, vm: &VM) {
        // logical_and > equality ( "and" equality )* ;

        // left side has already been compiled
        // so all we have to do is jump if the result was false
        // (short-circuit), otherwise finish parsing

        let end_idx = self.emit_instruction(OpCode::JumpIfFalse(0));
        self.emit_instruction(OpCode::Pop);

        self.parse_precedence(Precedence::And, vm);

        self.patch_jump(end_idx);
    }

    fn or(&mut self, _can_assign: bool, vm: &VM) {
        // logical_or -> logical_and ( "or" logical_and )* ;

        // left side has already been compiled
        // so all we have to do is jump if the result was true
        // - with an extra jump over that if the result was false -
        // (short-circuit), otherwise finish parsing
        // NOTE: this isn't the best way to do this, but the book
        // does it to demonstrate the flexibility we have with implementation

        let else_idx = self.emit_instruction(OpCode::JumpIfFalse(0));
        let end_idx = self.emit_instruction(OpCode::Jump(0));

        self.patch_jump(else_idx);
        self.emit_instruction(OpCode::Pop);

        self.parse_precedence(Precedence::Or, vm);

        self.patch_jump(end_idx);
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
        let (get, set) = match self.compiler.resolve_local(&name) {
            Ok(idx) => match idx {
                Some(idx) => (OpCode::GetLocal(idx), OpCode::SetLocal(idx)),
                None => {
                    // TODO: this adds the constant to the chunk
                    // even if it already exists, that's not great
                    let idx = self.identifier_constant(&name, vm);
                    (OpCode::GetGlobal(idx), OpCode::SetGlobal(idx))
                }
            },
            Err((idx, err)) => {
                self.error(err);
                (OpCode::GetLocal(idx), OpCode::SetLocal(idx))
            }
        };

        if can_assign && self.r#match(TokenType::Equal) {
            self.expression(vm);
            self.emit_instruction(set);
        } else {
            self.emit_instruction(get);
        }
    }

    fn variable(&mut self, can_assign: bool, vm: &VM) {
        let name = self.previous.borrow().lexeme.unwrap();
        self.named_variable(name, can_assign, vm);
    }

    fn literal(&mut self) {
        let token = self.previous.borrow().r#type;
        match token {
            TokenType::Nil => {
                self.emit_instruction(OpCode::Nil);
            }
            TokenType::False => {
                self.emit_instruction(OpCode::False);
            }
            TokenType::True => {
                self.emit_instruction(OpCode::True);
            }
            _ => unreachable!(),
        }
    }

    fn emit_instruction(&mut self, instruction: OpCode) -> usize {
        let line = self.previous.borrow().line;
        match self.compiler.function.as_ref() {
            Object::Function(f) => f.get_chunk().borrow_mut().write(instruction, line),
            _ => unreachable!(),
        }
    }

    fn emit_instructions(&mut self, instructions: impl AsRef<[OpCode]>) {
        for instruction in instructions.as_ref() {
            self.emit_instruction(*instruction);
        }
    }

    fn emit_loop(&mut self, idx: usize) {
        let offset = self.get_chunk_size() - idx;
        if offset > std::u16::MAX as usize {
            self.error("Loop body too large.");
        }

        self.emit_instruction(OpCode::Loop(offset as u16));
    }

    fn patch_jump(&mut self, idx: usize) {
        let jump = self.get_chunk_size() - idx;
        if jump > std::u16::MAX as usize {
            self.error("Too much code to jump over.");
        }

        match self.compiler.function.as_ref() {
            Object::Function(f) => f.get_chunk().borrow_mut().patch_jump(idx, jump as u16),
            _ => unreachable!(),
        }
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let idx = match self.compiler.function.as_ref() {
            Object::Function(f) => f.get_chunk().borrow_mut().add_constant(value),
            _ => unreachable!(),
        };

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

    fn end_compiler(mut self) -> Result<Rc<Function>, InterpretError> {
        self.emit_return();

        #[cfg(feature = "debug_code")]
        if !self.had_error() {
            self.disassemble_chunk(self.get_name());
        }

        if self.had_error() {
            Err(InterpretError::Compile)
        } else {
            Ok(self.compiler.function.as_function())
        }
    }

    /// Produce an error at the current token
    fn error_at_current(&self, message: impl AsRef<str>) {
        self.error_at(&self.current.borrow(), message);
    }

    /// Produce an error at the previous token
    fn error(&self, message: impl AsRef<str>) {
        self.error_at(&self.previous.borrow(), message);
    }

    /// Produce an error at the given token
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
                #[cfg(feature = "switch")]
                TokenType::Switch => return,
                #[cfg(feature = "extended_loops")]
                TokenType::Break | TokenType::Continue => return,
                _ => (),
            }

            self.advance();
        }
    }
}

/// Compiles lox source
pub fn compile(input: impl AsRef<str>, vm: &VM) -> Result<Rc<Function>, InterpretError> {
    let scanner = Scanner::new(input.as_ref());
    let mut parser = Parser::new(scanner, vm);

    // prime the parser
    parser.advance();

    // program -> declaration* EOF
    loop {
        if parser.r#match(TokenType::Eof) {
            break;
        }

        parser.declaration(vm);
    }

    parser.end_compiler()
}

#[cfg(test)]
mod tests {
    // TODO:
}
