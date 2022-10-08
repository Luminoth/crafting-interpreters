//! Lox source scanner

use std::cell::RefCell;

/// Lox token type
#[derive(
    Debug, Default, PartialEq, Eq, Copy, Clone, strum_macros::Display, strum_macros::AsRefStr,
)]
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
    //Break,
    //Continue,
    Nil,
    // TODO: #[cfg(not(feature = "native_print"))]
    Print,
    Return,
    Var,

    /// Scanning error
    Error,

    /// End of file
    #[default]
    Eof,
}

/// Lox token
#[derive(Debug, Copy, Clone, Default)]
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

    /// Scan and return the next token in the source stream
    pub fn scan_token(&self) -> Token<'a> {
        if let Some(error) = self.skip_whitespace_and_comments() {
            return error;
        }

        *self.start.borrow_mut() = self.current();

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

    #[inline]
    fn start(&self) -> usize {
        *self.start.borrow()
    }

    #[inline]
    fn peek_start(&self) -> char {
        self.source.as_bytes()[self.start()] as char
    }

    #[inline]
    fn peek_start_next(&self) -> char {
        self.source.as_bytes()[self.start() + 1] as char
    }

    #[inline]
    fn current(&self) -> usize {
        *self.current.borrow()
    }

    #[inline]
    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.as_bytes()[self.current()] as char
    }

    #[inline]
    fn peek_next(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.as_bytes()[self.current() + 1] as char
    }

    #[inline]
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

    #[inline]
    fn is_at_end(&self) -> bool {
        self.current() >= self.source.len()
    }

    // this can return an error token
    fn skip_whitespace_and_comments(&self) -> Option<Token<'a>> {
        loop {
            if self.is_at_end() {
                return None;
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

                        if self.is_at_end() {
                            return Some(self.error_token("Unterminated multi-line comment"));
                        }

                        // consume the closing '*/'
                        self.advance();
                        self.advance();
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }
    }

    fn string(&self) -> Token<'a> {
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

    fn identifier(&self) -> Token<'a> {
        loop {
            if !self.peek().is_ascii_alphabetic() && !self.peek().is_ascii_digit() {
                break;
            }

            self.advance();
        }

        self.make_token(self.identifier_type())
    }

    fn check_keyword(
        &self,
        keyword_start: usize,
        rest: impl AsRef<str>,
        r#type: TokenType,
    ) -> TokenType {
        let rest = rest.as_ref();

        if self.current() - self.start() == keyword_start + rest.len() {
            let start = self.start() + keyword_start;
            let source = &self.source[start..start + rest.len()];
            if source == rest {
                return r#type;
            }
        }

        TokenType::Identifier
    }

    fn identifier_type(&self) -> TokenType {
        match self.peek_start() {
            'a' => self.check_keyword(1, "nd", TokenType::And),
            'c' => self.check_keyword(1, "lass", TokenType::Class),
            'e' => self.check_keyword(1, "lse", TokenType::Else),
            'f' => {
                if self.current() - self.start() > 1 {
                    match self.peek_start_next() {
                        'a' => self.check_keyword(2, "lse", TokenType::False),
                        'o' => self.check_keyword(2, "r", TokenType::For),
                        'u' => self.check_keyword(2, "n", TokenType::Fun),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'i' => self.check_keyword(1, "f", TokenType::If),
            'n' => self.check_keyword(1, "il", TokenType::Nil),
            'o' => self.check_keyword(1, "r", TokenType::Or),
            // TODO: #[cfg(not(feature = "native_print"))]
            'p' => self.check_keyword(1, "rint", TokenType::Print),
            'r' => self.check_keyword(1, "eturn", TokenType::Return),
            's' => self.check_keyword(1, "uper", TokenType::Super),
            't' => {
                if self.current() - self.start() > 1 {
                    match self.peek_start_next() {
                        'h' => self.check_keyword(2, "is", TokenType::This),
                        'r' => self.check_keyword(2, "ue", TokenType::True),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'v' => self.check_keyword(1, "ar", TokenType::Var),
            'w' => self.check_keyword(1, "hile", TokenType::While),
            _ => TokenType::Identifier,
        }
    }

    fn number(&self) -> Token<'a> {
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

    #[inline]
    fn make_token(&self, r#type: TokenType) -> Token<'a> {
        Token {
            r#type,
            lexeme: Some(&self.source[self.start()..self.current()]),
            line: *self.line.borrow(),
        }
    }

    #[inline]
    fn error_token(&self, message: &'static str) -> Token<'a> {
        Token {
            r#type: TokenType::Error,
            lexeme: Some(message),
            line: *self.line.borrow(),
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO:
}
