//! Lox Virtual Machine

use std::cell::RefCell;
use std::collections::HashMap;
#[cfg(feature = "debug_trace")]
use std::fmt::Write;
use std::rc::Rc;

use thiserror::Error;
use tracing::{error, warn};

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

    /// Objects for garbage collection
    objects: RefCell<Vec<Rc<Object>>>,

    /// String table
    strings: RefCell<HashMap<u64, Rc<String>>>,

    /// Global variables
    globals: RefCell<HashMap<u64, Value>>,
}

impl Drop for VM {
    fn drop(&mut self) {
        self.free_objects()
    }
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

            objects: RefCell::new(Vec::new()),
            strings: RefCell::new(HashMap::new()),
            globals: RefCell::new(HashMap::new()),
        }
    }

    /// Adds an Object for garbage collection
    pub fn add_object(&self, object: Rc<Object>) {
        self.objects.borrow_mut().push(object);
    }

    /// Free all of the VM Objects
    pub fn free_objects(&self) {
        self.strings.borrow_mut().clear();

        // TODO: we need a hash set or something for this to actually work
        // I'm pretty sure the object list itself is over-holding references
        #[cfg(feature = "gc_leak_check")]
        for object in self.objects.borrow().iter() {
            let count = Rc::strong_count(object);
            if count > 1 {
                tracing::warn!("leaking {} object strong references", count);
            }

            let count = Rc::weak_count(object);
            if count > 0 {
                tracing::warn!("leaking {} object weak references", count);
            }
        }

        self.objects.borrow_mut().clear();
    }

    /// Looks up a string in the string table
    pub fn find_string(&self, hash: u64) -> Option<Rc<String>> {
        self.strings.borrow().get(&hash).cloned()
    }

    /// Adds a string to the string table
    pub fn add_string(&self, hash: u64, v: Rc<String>) {
        if self.strings.borrow_mut().insert(hash, v).is_some() {
            warn!("string table overwrite!");
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
    fn read_string<'a>(&self, chunk: &'a Chunk, idx: u8) -> (&'a String, u64) {
        let constant = chunk.get_constant(idx);
        constant.as_string()
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
                tracing::info!("{}", stack);

                instruction.disassemble("", &chunk);
            }

            match instruction {
                OpCode::Constant(idx) => {
                    let constant = chunk.get_constant(*idx);
                    self.push(constant.clone());
                }
                OpCode::DefineGlobal(idx) => {
                    // look up the variable name
                    let (_, hash) = self.read_string(&chunk, *idx);

                    // insert the variable into the globals
                    // peek and then pop to avoid the GC missing the value mid-operation
                    // Lox allows global overwrites
                    self.globals.borrow_mut().insert(hash, self.peek(0));
                    self.pop();
                }
                OpCode::GetGlobal(idx) => {
                    // look up the variable name
                    let (name, hash) = self.read_string(&chunk, *idx);

                    if let Some(value) = self.globals.borrow().get(&hash) {
                        // copy the value to the stack
                        self.push(value.clone());
                    } else {
                        self.runtime_error(format!("Undefined variable '{}'.", name));
                        return Err(InterpretError::Runtime);
                    }
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
                OpCode::Pop => {
                    self.pop();
                }
                OpCode::Print => println!("{}", self.pop()),
                OpCode::Return => return Ok(()),
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
