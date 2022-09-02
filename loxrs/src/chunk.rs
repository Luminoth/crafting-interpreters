//! Bytecode chunks

use crate::value::*;

/// Bytecode operation codes
#[derive(Debug, strum_macros::Display, strum_macros::AsRefStr, strum_macros::EnumCount)]
pub enum OpCode {
    /// A constant value
    #[strum(serialize = "OP_CONSTANT")]
    Constant(u8),

    /// Return from the current function
    #[strum(serialize = "OP_RETURN")]
    Return,
}

impl OpCode {
    /// Disassemble the opcode to stdout and return the "size" of the instruction
    pub fn disassemble(&self, chunk: &Chunk) -> usize {
        match self {
            Self::Constant(idx) => {
                println!(
                    "{:<16} {:0>4} '{}'",
                    self, idx, chunk.constants[*idx as usize]
                );
                2
            }
            Self::Return => {
                println!("{}", self);
                1
            }
        }
    }
}

/// Chunk of bytecode
#[derive(Debug)]
pub struct Chunk {
    /// Bytecode instructions
    code: Vec<OpCode>,

    /// Constants
    constants: ValueArray,
}

impl Chunk {
    /// Create a new chunk of bytecode
    pub fn new() -> Self {
        // 8 here to match what GROW_CAPACITY starts with
        Self {
            code: Vec::with_capacity(8),
            constants: ValueArray::with_capacity(8),
        }
    }

    /// Write an opcode to the chunk
    pub fn write(&mut self, opcode: OpCode) {
        self.code.push(opcode);
    }

    /// Add a constant to the chunk and return its index
    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        (self.constants.len() - 1) as u8
    }

    /// Disassemble the chunk to stdout
    pub fn disassemble(&self, name: impl AsRef<str>) {
        println!("== {} ==", name.as_ref());

        let mut offset = 0;
        for code in &self.code {
            print!("{:0>4} ", offset);
            offset += code.disassemble(self);
        }
    }
}
