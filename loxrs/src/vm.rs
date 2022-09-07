//! Lox Virtual Machine

use std::cell::RefCell;

use thiserror::Error;

use crate::chunk::*;
use crate::value::*;

const STACK_MAX: usize = 256;

/// Errors returned by the VM interpreter
#[derive(Error, Debug)]
pub enum InterpretError {
    /// A compile error
    #[error("compile error")]
    CompileError,

    /// A runtime error
    #[error("runtime error")]
    RuntimeError,
}

/// The Lox Virtual Machine
#[derive(Debug)]
pub struct VM<'a> {
    /// The chunk currently being processed
    chunk: RefCell<Option<&'a Chunk>>,

    /// Instruction pointer / program counter
    ip: RefCell<usize>,

    /// Value stack
    stack: RefCell<[Value; STACK_MAX]>,

    /// Stack pointer
    sp: RefCell<usize>,
}

impl<'a> VM<'a> {
    /// Creates a new VM
    pub fn new() -> Self {
        Self {
            chunk: RefCell::new(None),
            ip: RefCell::new(0),
            stack: RefCell::new([Value::default(); STACK_MAX]),
            sp: RefCell::new(0),
        }
    }

    /// Interprets a bytecode chunk
    pub fn interpret(&self, chunk: &'a Chunk) -> Result<(), InterpretError> {
        *self.chunk.borrow_mut() = Some(chunk);
        *self.ip.borrow_mut() = 0;

        self.run()
    }

    fn push(&self, value: Value) {
        let sp = *self.sp.borrow();
        self.stack.borrow_mut()[sp] = value;
        *self.sp.borrow_mut() += 1;
    }

    fn pop(&self) -> Value {
        let sp = *self.sp.borrow() - 1;
        let ret = self.stack.borrow()[sp];
        *self.sp.borrow_mut() -= 1;

        ret
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
            let instruction = self.read_byte();

            #[cfg(feature = "debug_trace")]
            {
                print!("          ");
                for slot in 0..*self.sp.borrow() {
                    print!("[ {} ]", self.stack.borrow()[slot]);
                }
                println!();
                instruction.disassemble(self.chunk.borrow().unwrap());
            }

            match instruction {
                OpCode::Constant(idx) => {
                    let constant = self.chunk.borrow().unwrap().get_constant(*idx);
                    self.push(*constant);
                }
                OpCode::Negate => {
                    self.push(-self.pop());
                }
                OpCode::Return => {
                    let value = self.pop();
                    println!("{}", value);
                    return Ok(());
                }
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
