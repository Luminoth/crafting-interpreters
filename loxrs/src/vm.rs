//! Lox Virtual Machine

use std::cell::RefCell;

use thiserror::Error;

use crate::chunk::*;

#[derive(Error, Debug)]
pub enum InterpretError {
    /// A compile error
    #[error("compile error")]
    CompileError,

    /// A runtime error
    #[error("runtime error")]
    RuntimeError,
}

#[derive(Debug)]
pub struct VM<'a> {
    /// The chunk currently being processed
    chunk: RefCell<Option<&'a Chunk>>,

    /// Instruction pointer / program counter
    ip: RefCell<usize>,
}

impl<'a> VM<'a> {
    /// Creates a new VM
    pub fn new() -> Self {
        Self {
            chunk: RefCell::new(None),
            ip: RefCell::new(0),
        }
    }

    /// Interprets a bytecode chunk
    pub fn interpret(&self, chunk: &'a Chunk) -> Result<(), InterpretError> {
        *self.chunk.borrow_mut() = Some(chunk);
        *self.ip.borrow_mut() = 0;

        self.run()
    }

    // READ_BYTE()
    #[inline]
    fn read_byte(&self) -> &OpCode {
        let ip = *self.ip.borrow();
        let ret = self.chunk.borrow().unwrap().read(ip);
        *self.ip.borrow_mut() += 1;

        ret
    }

    fn run(&self) -> Result<(), InterpretError> {
        loop {
            match self.read_byte() {
                OpCode::Constant(idx) => {
                    let constant = self.chunk.borrow().unwrap().get_constant(*idx);
                    println!("{}", constant);
                }
                OpCode::Return => return Ok(()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpret_basic() {
        let vm = VM::new();
        let mut chunk = Chunk::new();
        chunk.write(OpCode::Return, 123);
        assert_eq!(vm.interpret(&chunk).unwrap(), ());
    }
}
