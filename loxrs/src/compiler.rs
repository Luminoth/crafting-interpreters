//! Lox compiler

use crate::scanner::*;

/// Compiles lox source
pub async fn compile(input: impl AsRef<str>) {
    let scanner = Scanner::new(input.as_ref());
}
