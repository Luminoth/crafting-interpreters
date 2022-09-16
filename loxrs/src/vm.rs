//! Lox Virtual Machine

use std::cell::RefCell;
#[cfg(feature = "debug_trace")]
use std::fmt::Write;

use thiserror::Error;
use tracing::{error, info};

use crate::chunk::*;
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
pub struct VM {
    /// The chunk currently being processed
    chunk: RefCell<Chunk>,

    /// Instruction pointer / program counter
    ip: RefCell<usize>,

    /// Value stack
    stack: RefCell<Stack>,

    /// Stack pointer
    #[cfg(not(feature = "dynamic_stack"))]
    sp: RefCell<usize>,
}

impl VM {
    /// Creates a new VM
    pub fn new() -> Self {
        #[cfg(feature = "dynamic_stack")]
        let stack = Stack::with_capacity(STACK_MAX);
        #[cfg(not(feature = "dynamic_stack"))]
        let stack = [(); STACK_MAX].map(|_| Value::default());

        Self {
            chunk: RefCell::new(Chunk::new()),
            ip: RefCell::new(0),
            stack: RefCell::new(stack),

            #[cfg(not(feature = "dynamic_stack"))]
            sp: RefCell::new(0),
        }
    }

    /// Compile and interpret a Lox program
    pub fn interpret(&self, chunk: Chunk) -> Result<(), InterpretError> {
        *self.chunk.borrow_mut() = chunk;
        *self.ip.borrow_mut() = 0;

        // TODO: reset the stack?

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
        {
            self.stack.borrow_mut().pop().unwrap()
        }

        #[cfg(not(feature = "dynamic_stack"))]
        {
            let sp = *self.sp.borrow() - 1;
            let ret = self.stack.borrow()[sp].clone();
            *self.sp.borrow_mut() -= 1;

            ret
        }
    }

    fn peek(&self, distance: isize) -> Value {
        #[cfg(feature = "dynamic_stack")]
        {
            let sp = self.stack.borrow().len() as isize - 1 - distance;
            self.stack.borrow()[sp as usize].clone()
        }

        #[cfg(not(feature = "dynamic_stack"))]
        {
            let sp = *self.sp.borrow() as isize - 1 - distance;
            self.stack.borrow()[sp as usize].clone()
        }
    }

    #[inline]
    fn read_byte<'a>(&self, chunk: &'a Chunk) -> &'a OpCode {
        let ip = *self.ip.borrow();
        let ret = chunk.read(ip);
        *self.ip.borrow_mut() += 1;

        ret
    }

    #[inline]
    fn binary_op<C>(&self, op: C) -> Result<(), InterpretError>
    where
        C: FnOnce(Value, Value) -> Result<Value, InterpretError>,
    {
        let b = self.pop();
        let a = self.pop();
        self.push(op(a, b)?);

        Ok(())
    }

    fn run(&self) -> Result<(), InterpretError> {
        loop {
            let chunk = self.chunk.borrow();
            let instruction = self.read_byte(&chunk);

            #[cfg(feature = "debug_trace")]
            {
                let mut stack = String::from("          ");
                #[cfg(feature = "dynamic_stack")]
                for value in self.stack.borrow().iter() {
                    write!(stack, "[ {} ]", value).map_err(|_| InterpretError::Internal)?;
                }
                #[cfg(not(feature = "dynamic_stack"))]
                for slot in 0..*self.sp.borrow() {
                    write!(stack, "[ {} ]", self.stack.borrow()[slot])
                        .map_err(|_| InterpretError::Internal)?;
                }
                info!("{}", stack);

                instruction.disassemble("", &chunk);
            }

            match instruction {
                OpCode::Constant(idx) => {
                    let constant = chunk.get_constant(*idx);
                    self.push(constant.clone());
                }
                OpCode::Nil => self.push(Value::Nil),
                OpCode::False => self.push(false.into()),
                OpCode::True => self.push(true.into()),
                OpCode::Equal => self.binary_op(|a, b| Ok(a.equals(b)))?,
                #[cfg(feature = "extended_opcodes")]
                OpCode::NotEqual => self.binary_op(|a, b| Ok(a.not_equals(b)))?,
                OpCode::Less => self.binary_op(|a, b| a.less(b, self))?,
                #[cfg(feature = "extended_opcodes")]
                OpCode::LessEqual => self.binary_op(|a, b| a.less_equal(b, self))?,
                OpCode::Greater => self.binary_op(|a, b| a.greater(b, self))?,
                #[cfg(feature = "extended_opcodes")]
                OpCode::GreaterEqual => self.binary_op(|a, b| a.greater_equal(b, self))?,
                OpCode::Add => self.binary_op(|a, b| a.add(b, self))?,
                OpCode::Subtract => self.binary_op(|a, b| a.subtract(b, self))?,
                OpCode::Multiply => self.binary_op(|a, b| a.multiply(b, self))?,
                OpCode::Divide => self.binary_op(|a, b| a.divide(b, self))?,
                OpCode::Negate => {
                    let v = self.peek(0).negate(self)?;

                    self.pop();
                    self.push(v);
                }
                OpCode::Not => self.push(self.pop().is_falsey().into()),
                OpCode::Return => {
                    let value = self.pop();
                    info!("{}", value);
                    return Ok(());
                }
            }
        }
    }

    pub fn runtime_error(&self, message: impl AsRef<str>) {
        error!("{}", message.as_ref());
        error!(
            "[line {}] in script\n",
            self.chunk.borrow().get_line(*self.ip.borrow() - 1)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpret_basic() {
        let vm = VM::new();
        let mut chunk = Chunk::new();
        chunk.write(OpCode::Nil, 123);
        chunk.write(OpCode::Return, 123);
        assert_eq!(vm.interpret(chunk).unwrap(), ());
    }
}
