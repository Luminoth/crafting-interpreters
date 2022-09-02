//! Bytecode chunks

/// Bytecode operation codes
#[derive(Debug, strum_macros::Display, strum_macros::AsRefStr, strum_macros::EnumCount)]
pub enum OpCode {
    /// Return from the current function
    #[strum(serialize = "OP_RETURN")]
    Return,
}

impl OpCode {
    /// Disassemble the opcode to stdout
    pub fn disassemble(&self, offset: usize) -> usize {
        match self {
            Self::Return => {
                println!("{}", self);
                offset + 1
            }
        }
    }
}

/// Chunk of bytecode
#[derive(Debug)]
pub struct Chunk {
    /// Bytecode instructions
    code: Vec<OpCode>,
}

impl Chunk {
    /// Create a new chunk of bytecode
    pub fn new() -> Self {
        Self {
            // 8 here to match what GROW_CAPACITY starts with
            code: Vec::with_capacity(8),
        }
    }

    /// Write an opcode to the chunk
    pub fn write(&mut self, byte: OpCode) {
        self.code.push(byte);
    }

    /// Disassemble the chunk to stdout
    pub fn disassemble(&self, name: impl AsRef<str>) {
        println!("== {} ==", name.as_ref());

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:0>4} ", offset);
        self.code[offset].disassemble(offset)
    }
}
