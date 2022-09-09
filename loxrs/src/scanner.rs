//! Lox source scanner

/// Lox source scanner
#[derive(Debug)]
pub struct Scanner<'a> {
    /// The source being scanned
    source: &'a str,

    /// The start of the current lexeme
    start: usize,

    /// The current character
    current: usize,

    /// The current line
    line: usize,
}

impl<'a> Scanner<'a> {
    /// Creates a new scanner
    pub fn new(input: &'a str) -> Self {
        Self {
            source: input.as_ref(),
            start: 0,
            current: 0,
            line: 1,
        }
    }
}
