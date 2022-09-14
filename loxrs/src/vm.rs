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

    // READ_BYTE()
    #[inline]
    fn read_byte<'a>(&self, chunk: &'a Chunk) -> &'a OpCode {
        let ip = *self.ip.borrow();
        let ret = chunk.read(ip);
        *self.ip.borrow_mut() += 1;

        ret
    }

    #[inline]
    fn binary_op_number<C>(&self, op: C) -> Result<(), InterpretError>
    where
        C: FnOnce(f64, f64) -> Result<f64, InterpretError>,
    {
        let b = self.pop();
        let a = self.pop();

        self.push(Value::Number(match a {
            Value::Number(a) => match b {
                Value::Number(b) => op(a, b)?,
                _ => {
                    self.runtime_error("Operands must be numbers.");
                    return Err(InterpretError::Runtime);
                }
            },
            _ => {
                self.runtime_error("Operands must be numbers.");
                return Err(InterpretError::Runtime);
            }
        }));

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
                OpCode::False => self.push(Value::Bool(false)),
                OpCode::True => self.push(Value::Bool(true)),
                OpCode::Add => {
                    // TODO: concatenate strings
                    self.binary_op_number(|a, b| Ok(a + b))?;
                }
                OpCode::Subtract => {
                    self.binary_op_number(|a, b| Ok(a - b))?;
                }
                OpCode::Multiply => {
                    self.binary_op_number(|a, b| Ok(a * b))?;
                }
                OpCode::Divide => {
                    self.binary_op_number(|a, b| {
                        if b == 0.0 {
                            self.runtime_error("Illegal divide by zero.");
                            return Err(InterpretError::Runtime);
                        }
                        Ok(a / b)
                    })?;
                }
                OpCode::Negate => match self.peek(0) {
                    Value::Number(v) => {
                        self.pop();
                        self.push(Value::Number(-v));
                    }
                    _ => {
                        self.runtime_error("Operand must be a number.");
                        return Err(InterpretError::Runtime);
                    }
                },
                OpCode::Return => {
                    let value = self.pop();
                    info!("{}", value);
                    return Ok(());
                }
            }
        }
    }

    fn runtime_error(&self, message: impl AsRef<str>) {
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
        chunk.write(OpCode::Return, 123);
        assert_eq!(vm.interpret(chunk).unwrap(), ());
    }
}
