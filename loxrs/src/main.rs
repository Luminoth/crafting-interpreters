//! Rust implementation of clox from Crafting Interpreters - Robert Nystrom

#![deny(warnings)]

mod chunk;
mod compiler;
mod object;
mod options;
mod scanner;
mod value;
mod vm;

use std::path::Path;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::Level;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::FmtSubscriber;

use compiler::*;
use options::*;
use vm::*;

fn init_logging() -> anyhow::Result<()> {
    let stderr = std::io::stderr.with_max_level(Level::ERROR);

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .without_time()
        .with_level(false)
        .with_target(false)
        .map_writer(move |w| stderr.or_else(w))
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options: Options = argh::from_env();

    // TODO: make this not mutually exclusive
    if options.tracing {
        console_subscriber::init();
    } else {
        init_logging()?
    };

    if let Some(filepath) = options.filepath {
        run_file(filepath).await?;
    } else {
        repl().await?;
    }

    Ok(())
}

async fn repl() -> anyhow::Result<()> {
    // TODO: any IO errors in here should probably exit(74)
    // (and this function shouldn't return a Result)

    let mut stdout = tokio::io::stdout();
    let reader = BufReader::new(tokio::io::stdin());

    let mut lines = reader.lines();
    loop {
        stdout.write_all(b"> ").await?;
        stdout.flush().await?;

        if let Some(line) = lines.next_line().await? {
            // ignore any errors that come out of this
            // (tho we may want to exit(74) if it's an internal error)
            let _ = interpret(line).await;
        } else {
            stdout.write_all(b"\n").await?;
            break;
        }
    }

    Ok(())
}

async fn run_file(filepath: impl AsRef<Path>) -> anyhow::Result<()> {
    // TODO: this should exit(74) if we fail to read the file
    // (and this function shouldn't return a Result)
    let source = tokio::fs::read_to_string(filepath).await?;

    match interpret(source).await {
        Ok(_) => Ok(()),
        Err(err) => match err {
            InterpretError::Internal => std::process::exit(1),
            InterpretError::Compile => std::process::exit(65),
            InterpretError::Runtime(_) => std::process::exit(70),
        },
    }
}

async fn interpret(input: String) -> Result<(), InterpretError> {
    tokio::task::spawn_blocking(move || {
        let vm = VM::new();
        let main = compile(input, &vm)?;

        vm.interpret(main)
    })
    .await
    .map_err(|_| InterpretError::Internal)
    .and_then(|result| result)
}
