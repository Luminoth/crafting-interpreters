//! Lox bytecode chunks

use tracing::info;

use crate::value::*;

/// Bytecode operation codes
// TODO: this is currently slightly less memory efficient than the book's implementation
// probably want to keep an eye on things to see if it gets really bad
#[derive(Debug, Copy, Clone, PartialEq, Eq, strum_macros::Display, strum_macros::AsRefStr)]
pub enum OpCode {
    /// A constant value
    #[strum(serialize = "OP_CONSTANT")]
    Constant(u8),

    /// A nil value
    #[strum(serialize = "OP_NIL")]
    Nil,

    /// A true value
    #[strum(serialize = "OP_TRUE")]
    True,

    /// A false value
    #[strum(serialize = "OP_FALSE")]
    False,

    /// Equality
    #[strum(serialize = "OP_EQUAL")]
    Equal,

    /// Not equality
    #[cfg(feature = "extended_opcodes")]
    #[strum(serialize = "OP_NOT_EQUAL")]
    NotEqual,

    /// Greater than
    #[strum(serialize = "OP_GREATER")]
    Greater,

    /// Greater than or equal
    #[cfg(feature = "extended_opcodes")]
    #[strum(serialize = "OP_GREATER_EQUAL")]
    GreaterEqual,

    /// Less than
    #[strum(serialize = "OP_LESS")]
    Less,

    /// Less than or equal
    #[cfg(feature = "extended_opcodes")]
    #[strum(serialize = "OP_LESS_EQUAL")]
    LessEqual,

    /// Addition
    #[strum(serialize = "OP_ADD")]
    Add,

    /// Subtraction
    #[strum(serialize = "OP_SUBTRACT")]
    Subtract,

    /// Multiplication
    #[strum(serialize = "OP_MULTIPLY")]
    Multiply,

    /// Division
    #[strum(serialize = "OP_DIVIDE")]
    Divide,

    /// Unary negation
    #[strum(serialize = "OP_NEGATE")]
    Negate,

    /// Logical not
    #[strum(serialize = "OP_NOT")]
    Not,

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
            _ => 1,
        }
    }

    /// Disassemble the opcode to stdout
    pub fn disassemble(&self, header: impl AsRef<str>, chunk: &Chunk) {
        match self {
            Self::Constant(idx) => {
                info!(
                    "{}{:<16} {:>4} '{}'",
                    header.as_ref(),
                    self,
                    idx,
                    chunk.constants[*idx as usize]
                );
            }
            _ => {
                info!("{}{}", header.as_ref(), self);
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

    /// The number of opcodes in the chunk
    pub fn size(&self) -> usize {
        self.code.len()
    }

    /// Reads the instruction at ip
    #[inline]
    pub fn read(&self, ip: usize) -> &OpCode {
        &self.code[ip]
    }

    #[inline]
    pub fn get_line(&self, ip: usize) -> usize {
        self.lines[ip]
    }

    /// Write an opcode to the chunk
    pub fn write(&mut self, opcode: OpCode, line: usize) {
        self.code.push(opcode);
        self.lines.push(line);
    }

    /// Gets the constant at the given index
    #[inline]
    pub fn get_constant(&self, idx: u8) -> &Value {
        &self.constants[idx as usize]
    }

    /// Add a constant to the chunk and return its index
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    /// Disassemble the chunk to stdout
    pub fn disassemble(&self, name: impl AsRef<str>) {
        info!("== {} ==", name.as_ref());

        let mut offset = 0;
        for (idx, code) in self.code.iter().enumerate() {
            let header = format!(
                "{:0>4} {}",
                offset,
                if offset > 0 && self.lines[idx] == self.lines[idx - 1] {
                    "   | ".to_owned()
                } else {
                    format!("{:>4} ", self.lines[idx])
                }
            );
            code.disassemble(header, self);
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
        assert_eq!(*chunk.read(0), OpCode::Return);
        assert_eq!(chunk.lines[0], 123);
    }

    #[test]
    fn test_constant() {
        let mut chunk = Chunk::new();

        let constant = chunk.add_constant(Value::Number(1.2));
        chunk.write(OpCode::Constant(constant as u8), 123);
        let idx = constant as usize;
        assert_eq!(chunk.code[idx], OpCode::Constant(0));
        assert_eq!(chunk.lines[idx], 123);
        assert_eq!(chunk.constants[idx], Value::Number(1.2));
        assert_eq!(*chunk.get_constant(idx as u8), Value::Number(1.2));

        let constant = chunk.add_constant(Value::Number(2.1));
        chunk.write(OpCode::Constant(constant as u8), 124);
        let idx = constant as usize;
        assert_eq!(chunk.code[idx], OpCode::Constant(1));
        assert_eq!(chunk.lines[idx], 124);
        assert_eq!(chunk.constants[idx], Value::Number(2.1));
        assert_eq!(*chunk.get_constant(idx as u8), Value::Number(2.1));
    }
}
