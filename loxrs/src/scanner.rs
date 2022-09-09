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
        self.skip_whitespace_and_comments();

        *self.start.borrow_mut() = *self.current.borrow();

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let ch = self.advance();

        // literals
        if ch.is_ascii_alphabetic() {
            return self.identifier();
        }

        if ch.is_ascii_digit() {
            return self.number();
        }

        match ch {
            // single character tokens
            '(' => self.make_token(TokenType::LeftParen),
            ')' => self.make_token(TokenType::RightParen),
            '{' => self.make_token(TokenType::LeftBrace),
            '}' => self.make_token(TokenType::RightBrace),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '-' => self.make_token(TokenType::Minus),
            '+' => self.make_token(TokenType::Plus),
            ';' => self.make_token(TokenType::Semicolon),
            '/' => self.make_token(TokenType::Slash),
            '*' => self.make_token(TokenType::Star),
            '?' => self.make_token(TokenType::Question),
            ':' => self.make_token(TokenType::Colon),

            // one or two character tokens
            '!' => self.make_token(if self.r#match('=') {
                TokenType::BangEqual
            } else {
                TokenType::Bang
            }),
            '=' => self.make_token(if self.r#match('=') {
                TokenType::EqualEqual
            } else {
                TokenType::Equal
            }),
            '<' => self.make_token(if self.r#match('=') {
                TokenType::LessEqual
            } else {
                TokenType::Less
            }),
            '>' => self.make_token(if self.r#match('=') {
                TokenType::GreaterEqual
            } else {
                TokenType::Greater
            }),

            // iterals
            '"' => self.string(),

            _ => self.error_token("Unexpected character."),
        }
    }

    fn peek(&self) -> char {
        let current = *self.current.borrow();
        self.source.as_bytes()[current] as char
    }

    fn peek_next(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        let next = *self.current.borrow() + 1;
        self.source.as_bytes()[next] as char
    }

    fn advance(&self) -> char {
        let current = self.peek();
        *self.current.borrow_mut() += 1;
        current
    }

    fn r#match(&self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.peek() != expected {
            return false;
        }

        *self.current.borrow_mut() += 1;
        true
    }

    fn is_at_end(&self) -> bool {
        *self.current.borrow() >= self.source.len()
    }

    fn skip_whitespace_and_comments(&self) {
        loop {
            if self.is_at_end() {
                return;
            }

            let ch = self.peek();
            match ch {
                // whitespace
                ' ' | '\r' | '\t' => _ = self.advance(),
                '\n' => {
                    *self.line.borrow_mut() += 1;
                    self.advance();
                }

                // comments
                '/' => {
                    if self.peek_next() == '/' {
                        // single-line comment
                        loop {
                            let ch = self.peek();
                            if ch == '\n' || self.is_at_end() {
                                break;
                            }
                            self.advance();
                        }
                    } else if self.peek_next() == '*' {
                        // multi-line comment
                        loop {
                            let ch = self.peek();
                            if (ch == '*' && self.peek_next() == '/') || self.is_at_end() {
                                break;
                            }

                            if ch == '\n' {
                                *self.line.borrow_mut() += 1
                            }

                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => break,
            }
        }
    }

    fn string(&self) -> Token {
        // TODO: handle escape characters

        loop {
            if self.peek() == '"' || self.is_at_end() {
                break;
            }

            // allow multiline strings
            if self.peek() == '\n' {
                *self.line.borrow_mut() += 1
            }

            self.advance();
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string.");
        }

        // consume closing '"'
        self.advance();

        self.make_token(TokenType::String)
    }

    fn identifier(&self) -> Token {
        loop {
            if !self.peek().is_ascii_alphabetic() && !self.peek().is_ascii_digit() {
                break;
            }

            self.advance();
        }

        self.make_token(self.identifier_type())
    }

    fn identifier_type(&self) -> TokenType {
        TokenType::Identifier
    }

    fn number(&self) -> Token {
        loop {
            if !self.peek().is_ascii_digit() {
                break;
            }

            self.advance();
        }

        // check for fractional
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // consume the '.'
            self.advance();

            loop {
                if !self.peek().is_ascii_digit() {
                    break;
                }

                self.advance();
            }
        }

        self.make_token(TokenType::Number)
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
