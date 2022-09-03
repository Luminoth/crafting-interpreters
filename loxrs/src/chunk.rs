//! Bytecode chunks

use crate::value::*;

/// Bytecode operation codes
#[derive(
    Debug, PartialEq, Eq, strum_macros::Display, strum_macros::AsRefStr, strum_macros::EnumCount,
)]
pub enum OpCode {
    /// A constant value
    #[strum(serialize = "OP_CONSTANT")]
    Constant(u8),

    /// Return from the current function
    #[strum(serialize = "OP_RETURN")]
    Return,
}

impl OpCode {
    /// Returns the "size" of the instruction
    ///
    /// Operands are stored with opcodes here
    /// but we still need to mirror the right offset in disassembly
    pub fn size(&self) -> usize {
        match self {
            Self::Constant(_) => 2,
            Self::Return => 1,
        }
    }

    /// Disassemble the opcode to stdout
    pub fn disassemble(&self, chunk: &Chunk) {
        match self {
            Self::Constant(idx) => {
                println!(
                    "{:<16} {:>4} '{}'",
                    self, idx, chunk.constants[*idx as usize]
                );
            }
            Self::Return => {
                println!("{}", self);
            }
        }
    }
}

/// Chunk of bytecode
#[derive(Debug)]
pub struct Chunk {
    /// Bytecode instructions
    code: Vec<OpCode>,

    /// Line numbers associated with each instruction
    lines: Vec<usize>,

    /// Constants
    constants: ValueArray,
}

impl Chunk {
    /// Create a new chunk of bytecode
    pub fn new() -> Self {
        // 8 here to match what GROW_CAPACITY starts with
        Self {
            code: Vec::with_capacity(8),
            lines: Vec::with_capacity(8),
            constants: ValueArray::with_capacity(8),
        }
    }

    /// Write an opcode to the chunk
    pub fn write(&mut self, opcode: OpCode, line: usize) {
        self.code.push(opcode);
        self.lines.push(line);
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
        for (idx, code) in self.code.iter().enumerate() {
            print!("{:0>4} ", offset);
            if offset > 0 && self.lines[idx] == self.lines[idx - 1] {
                print!("   | ");
            } else {
                print!("{:>4} ", self.lines[idx]);
            }
            code.disassemble(self);
            offset += code.size();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write() {
        let mut chunk = Chunk::new();
        chunk.write(OpCode::Return, 123);
        assert_eq!(chunk.code[0], OpCode::Return);
        assert_eq!(chunk.lines[0], 123);
    }

    #[test]
    fn test_constant() {
        let mut chunk = Chunk::new();

        let constant = chunk.add_constant(1.2);
        chunk.write(OpCode::Constant(constant), 123);
        let idx = constant as usize;
        assert_eq!(chunk.code[idx], OpCode::Constant(0));
        assert_eq!(chunk.lines[idx], 123);
        assert_eq!(chunk.constants[idx], 1.2);

        let constant = chunk.add_constant(2.1);
        chunk.write(OpCode::Constant(constant), 124);
        let idx = constant as usize;
        assert_eq!(chunk.code[idx], OpCode::Constant(1));
        assert_eq!(chunk.lines[idx], 124);
        assert_eq!(chunk.constants[idx], 2.1);
    }
}
