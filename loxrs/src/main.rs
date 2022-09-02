//! Rust implementation of clox from Crafting Interpreters - Robert Nystrom

mod chunk;
mod value;

use chunk::*;

fn main() {
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant(constant));
    chunk.write(OpCode::Return);
    chunk.disassemble("test chunk");
}
