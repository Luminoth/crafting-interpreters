//! Rust implementation of clox from Crafting Interpreters - Robert Nystrom

mod chunk;
mod value;
mod vm;

use chunk::*;
use vm::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant(constant), 123);

    let constant = chunk.add_constant(3.4);
    chunk.write(OpCode::Constant(constant), 123);

    chunk.write(OpCode::Add, 123);

    let constant = chunk.add_constant(5.6);
    chunk.write(OpCode::Constant(constant), 123);

    chunk.write(OpCode::Divide, 123);
    chunk.write(OpCode::Negate, 123);

    chunk.write(OpCode::Return, 123);

    chunk.disassemble("test chunk");

    tokio::task::spawn_blocking(move || {
        let vm = VM::new();
        vm.interpret(&chunk)
    })
    .await??;

    Ok(())
}
