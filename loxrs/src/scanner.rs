//! Lox source scanner

use std::cell::RefCell;

/// Lox token type
#[derive(Debug, PartialEq, Eq, Copy, Clone, strum_macros::Display)]
pub enum TokenType {
    // single character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Question,
    Colon,

    // one or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // literals
    Identifier,
    String,
    Number,

    // keywords
    And,
    Or,
    If,
    Else,
    Class,
    Super,
    This,
    True,
    False,
    Fun,
    For,
    While,
    Break,
    Continue,
    Nil,
    Print,
    Return,
    Var,

    /// Scanning error
    Error,

    /// End of file
    Eof,
}

/// Lox token
#[derive(Debug)]
pub struct Token<'a> {
    /// The token type
    pub r#type: TokenType,

    /// The token lexeme
    pub lexeme: Option<&'a str>,

    /// The line the token is from
    pub line: usize,
}

/// Lox source scanner
#[derive(Debug)]
pub struct Scanner<'a> {
    /// The source being scanned
    source: &'a str,

    /// The start of the current lexeme
    start: RefCell<usize>,

    /// The current character
    current: RefCell<usize>,

    /// The current line
    line: RefCell<usize>,
}

impl<'a> Scanner<'a> {
    /// Creates a new scanner
    pub fn new(input: &'a str) -> Self {
        Self {
            source: input,
            start: RefCell::new(0),
            current: RefCell::new(0),
            line: RefCell::new(1),
        }
    }

    pub fn scan_token(&self) -> Token {
        *self.start.borrow_mut() = *self.current.borrow();

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        self.error_token("Unexpected character.")
    }

    fn is_at_end(&self) -> bool {
        *self.current.borrow() >= self.source.len()
    }

    fn make_token(&self, r#type: TokenType) -> Token {
        let start = *self.start.borrow();
        let current = *self.current.borrow();
        Token {
            r#type,
            lexeme: Some(&self.source[start..current]),
            line: *self.line.borrow(),
        }
    }

    fn error_token(&self, message: &'static str) -> Token {
        Token {
            r#type: TokenType::Error,
            lexeme: Some(message),
            line: *self.line.borrow(),
        }
    }
}
