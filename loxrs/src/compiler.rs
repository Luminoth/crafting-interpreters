//! Lox compiler

use tracing::info;

use crate::chunk::*;
use crate::scanner::*;

/// Compiles lox source
pub fn compile(input: String) -> Option<Chunk> {
    let chunk = Chunk::new();

    let scanner = Scanner::new(&input);

    let mut line = -1;
    loop {
        let token = scanner.scan_token();
        info!(
            "{} {:>2} '{}'",
            if token.line as isize != line {
                let f = format!("{:>4} ", token.line);
                line = token.line as isize;
                f
            } else {
                "   | ".to_owned()
            },
            token.r#type as u8,
            token.lexeme.unwrap_or_default()
        );

        if token.r#type == TokenType::Eof || token.r#type == TokenType::Error {
            break;
        }
    }

    Some(chunk)
}

#[cfg(test)]
mod tests {
    // TODO:
}
