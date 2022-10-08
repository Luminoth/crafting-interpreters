//! Lox bytecode chunks

use crate::value::*;

/// Bytecode operation codes
// TODO: this is currently slightly less memory efficient than the book's implementation
// probably want to keep an eye on things to see if it gets really bad
#[derive(Debug, Copy, Clone, PartialEq, Eq, strum_macros::Display, strum_macros::AsRefStr)]
pub enum OpCode {
    /// A constant value
    #[strum(serialize = "OP_CONSTANT")]
    Constant(u8),

    /// Global variable declaration
    #[strum(serialize = "OP_DEFINE_GLOBAL")]
    DefineGlobal(u8),

    /// Get a local variable value
    #[strum(serialize = "OP_GET_LOCAL")]
    GetLocal(u8),

    /// Set a local variable value
    #[strum(serialize = "OP_SET_LOCAL")]
    SetLocal(u8),

    /// Get a global variable value
    #[strum(serialize = "OP_GET_GLOBAL")]
    GetGlobal(u8),

    /// Set a global variable value
    #[strum(serialize = "OP_SET_GLOBAL")]
    SetGlobal(u8),

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

    /// Pop the top of the stack and discard it
    #[strum(serialize = "OP_POP")]
    Pop,

    /// Print a value
    #[strum(serialize = "OP_PRINT")]
    Print,

    /// Jump to an offset
    #[strum(serialize = "OP_JUMP")]
    Jump(u16),

    /// Jump to an offset if the top of the stack is falsey
    #[strum(serialize = "OP_JUMP_IF_FALSE")]
    JumpIfFalse(u16),

    /// Loop back by an offset
    #[strum(serialize = "OP_LOOP")]
    Loop(u16),

    /// Return from the current function / program
    #[strum(serialize = "OP_RETURN")]
    Return,
}

impl OpCode {
    /// Returns the "size" of the instruction in bytes
    ///
    /// Operands are stored with opcodes here
    /// but we still need to mirror the right offset in disassembly
    #[cfg(any(feature = "debug_code", feature = "debug_trace"))]
    pub fn size(&self) -> usize {
        /*match self {
            Self::Constant(_)
            | Self::DefineGlobal(_)
            | Self::GetLocal(_)
            | Self::SetLocal(_)
            | Self::GetGlobal(_)
            | Self::SetGlobal(_) => 2,
            Self::Jump(_) | Self::JumpIfFalse(_) | Self::Loop(_) => 3,
            _ => 1,
        }*/

        // TODO: the output is unreadable as bytes now that jump is supported
        // so for now just do it per-opcode
        1
    }

    /// Disassemble the opcode to stdout
    #[cfg(any(feature = "debug_code", feature = "debug_trace"))]
    pub fn disassemble(&self, header: impl AsRef<str>, chunk: &Chunk, ip: usize) {
        match self {
            // constant instructions
            Self::Constant(idx)
            | Self::DefineGlobal(idx)
            | Self::GetGlobal(idx)
            | Self::SetGlobal(idx) => {
                tracing::info!(
                    "{}{:<16} {:>4} '{}'",
                    header.as_ref(),
                    self,
                    idx,
                    chunk.constants[*idx as usize]
                );
            }
            // jump instructions
            Self::Jump(offset) | Self::JumpIfFalse(offset) => {
                tracing::info!(
                    "{}{:<16} {:0>4} -> {:0>4}",
                    header.as_ref(),
                    self,
                    ip,
                    ip + *offset as usize
                );
            }
            Self::Loop(offset) => {
                tracing::info!(
                    "{}{:<16} {:0>4} -> {:0>4}",
                    header.as_ref(),
                    self,
                    ip,
                    ip - *offset as usize
                );
            }
            // byte instructions
            Self::GetLocal(idx) | Self::SetLocal(idx) => {
                tracing::info!("{}{:<16} {:>4}", header.as_ref(), self, idx);
            }
            // simple instructions
            _ => {
                tracing::info!("{}{}", header.as_ref(), self);
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
    #[inline]
    pub fn size(&self) -> usize {
        self.code.len()
    }

    /// Reads the instruction at ip
    #[inline]
    pub fn read(&self, ip: usize) -> &OpCode {
        self.code.get(ip).unwrap()
    }

    #[inline]
    pub fn get_line(&self, ip: usize) -> usize {
        *self.lines.get(ip).unwrap()
    }

    /// Write an opcode to the chunk
    ///
    /// Returns the index of the opcode
    #[inline]
    pub fn write(&mut self, opcode: OpCode, line: usize) -> usize {
        self.code.push(opcode);
        self.lines.push(line);

        self.code.len() - 1
    }

    /// Patch the offset of the jump instruction at idx
    #[inline]
    pub fn patch_jump(&mut self, idx: usize, offset: u16) {
        match self.code.get_mut(idx).unwrap() {
            OpCode::Jump(v) | OpCode::JumpIfFalse(v) => *v = offset,
            _ => unreachable!(),
        }
    }

    /// Gets the constant at the given index
    #[inline]
    pub fn get_constant(&self, idx: u8) -> &Value {
        &self.constants[idx as usize]
    }

    pub fn free_constants(&mut self) {
        self.constants.clear();
    }

    /// Add a constant to the chunk and return its index
    #[inline]
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    /// Disassemble the chunk to stdout
    #[cfg(any(feature = "debug_code", feature = "debug_trace"))]
    pub fn disassemble(&self, name: impl AsRef<str>) {
        tracing::info!("== {} ==", name.as_ref());

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
            code.disassemble(header, self, idx);
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

        let constant = chunk.add_constant(1.2.into());
        chunk.write(OpCode::Constant(constant as u8), 123);
        let idx = constant as usize;
        assert_eq!(chunk.code[idx], OpCode::Constant(0));
        assert_eq!(chunk.lines[idx], 123);
        assert_eq!(chunk.constants[idx].equals(1.2.into()), true.into());
        assert_eq!(
            chunk.get_constant(idx as u8).equals(1.2.into()),
            true.into()
        );

        let constant = chunk.add_constant(2.1.into());
        chunk.write(OpCode::Constant(constant as u8), 124);
        let idx = constant as usize;
        assert_eq!(chunk.code[idx], OpCode::Constant(1));
        assert_eq!(chunk.lines[idx], 124);
        assert_eq!(chunk.constants[idx].equals(2.1.into()), true.into());
        assert_eq!(
            chunk.get_constant(idx as u8).equals(2.1.into()),
            true.into()
        );
    }
}
