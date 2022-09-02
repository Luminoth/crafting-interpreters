//! Rust implementation of clox from Crafting Interpreters - Robert Nystrom

mod chunk;

use chunk::*;

fn main() {
    let mut chunk = Chunk::new();
    chunk.write(OpCode::Return);
    chunk.disassemble("test chunk");
}
