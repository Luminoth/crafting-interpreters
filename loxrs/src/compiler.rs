//! Lox compiler

use crate::scanner::*;

/// Compiles lox source
pub async fn compile(input: String) {
    let _ = tokio::task::spawn_blocking(move || {
        let scanner = Scanner::new(&input);

        let mut line = -1;
        loop {
            let token = scanner.scan_token();
            if token.line as isize != line {
                print!("{:>4} ", token.line);
                line = token.line as isize;
            } else {
                print!("   | ");
            }

            println!(
                "{:>2} '{}'",
                token.r#type as u8,
                token.lexeme.unwrap_or_default()
            );

            if token.r#type == TokenType::Eof || token.r#type == TokenType::Error {
                break;
            }
        }
    })
    .await;
}
