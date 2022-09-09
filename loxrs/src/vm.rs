//! Lox Virtual Machine

use std::cell::RefCell;

use thiserror::Error;

use crate::chunk::*;
use crate::compiler::*;
use crate::value::*;

const STACK_MAX: usize = 256;

#[cfg(feature = "dynamic_stack")]
type Stack = Vec<Value>;
#[cfg(not(feature = "dynamic_stack"))]
type Stack = [Value; STACK_MAX];

/// Errors returned by the VM interpreter
#[derive(Error, Debug)]
pub enum InterpretError {
    /// An internal error
    #[error("internal error")]
    Internal,

    /// A compile error
    #[error("compile error")]
    Compile,

    /// A runtime error
    #[error("runtime error")]
    Runtime,
}

/// The Lox Virtual Machine
#[derive(Debug)]
pub struct VM<'a> {
    /// The chunk currently being processed
    chunk: RefCell<Option<&'a Chunk>>,

    /// Instruction pointer / program counter
    ip: RefCell<usize>,

    /// Value stack
    stack: RefCell<Stack>,

    /// Stack pointer
    #[cfg(not(feature = "dynamic_stack"))]
    sp: RefCell<usize>,
}

impl<'a> VM<'a> {
    /// Creates a new VM
    pub fn new() -> Self {
        #[cfg(feature = "dynamic_stack")]
        let stack = Stack::with_capacity(STACK_MAX);
        #[cfg(not(feature = "dynamic_stack"))]
        let stack = [Value::default(); STACK_MAX];

        Self {
            chunk: RefCell::new(None),
            ip: RefCell::new(0),
            stack: RefCell::new(stack),

            #[cfg(not(feature = "dynamic_stack"))]
            sp: RefCell::new(0),
        }
    }

    /// Interprets a bytecode chunk
    pub fn interpret(&self, chunk: &'a Chunk) -> Result<(), InterpretError> {
        *self.chunk.borrow_mut() = Some(chunk);
        *self.ip.borrow_mut() = 0;

        // TODO: maybe this should be an error?
        if self.chunk.borrow().unwrap().size() < 1 {
            return Ok(());
        }

        self.run()
    }

    fn push(&self, value: Value) {
        #[cfg(feature = "dynamic_stack")]
        self.stack.borrow_mut().push(value);

        #[cfg(not(feature = "dynamic_stack"))]
        {
            let sp = *self.sp.borrow();
            self.stack.borrow_mut()[sp] = value;
            *self.sp.borrow_mut() += 1;
        }
    }

    fn pop(&self) -> Value {
        #[cfg(feature = "dynamic_stack")]
        return self.stack.borrow_mut().pop().unwrap();

        #[cfg(not(feature = "dynamic_stack"))]
        {
            let sp = *self.sp.borrow() - 1;
            let ret = self.stack.borrow()[sp];
            *self.sp.borrow_mut() -= 1;

            ret
        }
    }

    // READ_BYTE()
    #[inline]
    fn read_byte(&self) -> &OpCode {
        let ip = *self.ip.borrow();
        let ret = self.chunk.borrow().unwrap().read(ip);
        *self.ip.borrow_mut() += 1;

        ret
    }

    #[inline]
    fn binary_op<C>(&self, op: C)
    where
        C: FnOnce(Value, Value) -> Value,
    {
        let b = self.pop();
        let a = self.pop();
        self.push(op(a, b));
    }

    fn run(&self) -> Result<(), InterpretError> {
        loop {
            let instruction = self.read_byte();

            #[cfg(feature = "debug_trace")]
            {
                print!("          ");
                #[cfg(feature = "dynamic_stack")]
                for value in self.stack.borrow().iter() {
                    print!("[ {} ]", value);
                }
                #[cfg(not(feature = "dynamic_stack"))]
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
                OpCode::Add => {
                    self.binary_op(|a, b| a + b);
                }
                OpCode::Subtract => {
                    self.binary_op(|a, b| a - b);
                }
                OpCode::Multiply => {
                    self.binary_op(|a, b| a * b);
                }
                OpCode::Divide => {
                    // TODO: divide by 0 error
                    self.binary_op(|a, b| a / b);
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

/// Compile and interpret lox source
pub async fn interpret(input: String) -> Result<(), InterpretError> {
    compile(input).await;

    tokio::task::spawn_blocking(move || {
        let chunk = Chunk::new();

        let vm = VM::new();
        vm.interpret(&chunk)
    })
    .await
    .map_err(|_| InterpretError::Internal)
    .and_then(|result| result)
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
