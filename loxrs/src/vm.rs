//! Lox Virtual Machine

use std::cell::RefCell;
use std::collections::HashMap;
#[cfg(feature = "debug_trace")]
use std::fmt::Write;
use std::rc::Rc;

use thiserror::Error;
use tracing::{debug, error, warn};

use crate::chunk::*;
use crate::compiler::LOCALS_MAX;
use crate::object::*;
use crate::value::*;

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
    Runtime(&'static str),
}

/// Lox function call stack frame
#[derive(Debug, Default)]
pub struct CallFrame {
    /// The function being interpreted
    func: Option<Rc<Function>>,

    /// Pointer to the caller's instruction pointer
    ip: usize,

    /// The function's base pointer in the VM stack
    bp: usize,
}

impl CallFrame {
    /// Free all of the frame Objects
    pub fn free_objects(&self) {
        if let Some(func) = self.func.as_ref() {
            let mut chunk = func.get_chunk().borrow_mut();
            chunk.free_constants();
        }
    }
}

const FRAMES_MAX: usize = 64;

#[cfg(feature = "dynamic_frames")]
type CallFrames = Vec<CallFrame>;
#[cfg(not(feature = "dynamic_frames"))]
type CallFrames = [CallFrame; FRAMES_MAX];

const STACK_MAX: usize = FRAMES_MAX * LOCALS_MAX;

#[cfg(feature = "dynamic_stack")]
type Stack = Vec<Value>;
#[cfg(not(feature = "dynamic_stack"))]
type Stack = [Value; STACK_MAX];

/// The Lox Virtual Machine
#[derive(Debug)]
pub struct VM {
    /// Call frame stack
    frames: RefCell<CallFrames>,

    /// Stack pointer
    #[cfg(not(feature = "dynamic_frames"))]
    frame_count: RefCell<usize>,

    /// Value stack
    stack: RefCell<Stack>,

    /// Stack pointer
    #[cfg(not(feature = "dynamic_stack"))]
    sp: RefCell<usize>,

    /// String table
    strings: RefCell<HashMap<u64, Rc<String>>>,

    /// Global variables
    globals: RefCell<HashMap<u64, Value>>,

    /// Objects for garbage collection
    objects: RefCell<Vec<Rc<Object>>>,
}

impl Drop for VM {
    fn drop(&mut self) {
        self.free_objects()
    }
}

impl VM {
    /// Creates a new VM
    pub fn new() -> Self {
        #[cfg(feature = "dynamic_frames")]
        let frames = Frames::with_capacity(FRAMES_MAX);
        #[cfg(not(feature = "dynamic_frames"))]
        let frames = [(); FRAMES_MAX].map(|_| CallFrame::default());

        #[cfg(feature = "dynamic_stack")]
        let stack = Stack::with_capacity(STACK_MAX);
        #[cfg(not(feature = "dynamic_stack"))]
        let stack = [(); STACK_MAX].map(|_| Value::default());

        Self {
            frames: RefCell::new(frames),

            #[cfg(not(feature = "dynamic_frames"))]
            frame_count: RefCell::new(0),

            stack: RefCell::new(stack),

            #[cfg(not(feature = "dynamic_stack"))]
            sp: RefCell::new(0),

            strings: RefCell::new(HashMap::new()),
            globals: RefCell::new(HashMap::new()),
            objects: RefCell::new(Vec::new()),
        }
    }

    /// Adds an Object for garbage collection
    pub fn add_object(&self, object: Rc<Object>) {
        self.objects.borrow_mut().push(object);
    }

    /// Free all of the VM Objects
    pub fn free_objects(&self) {
        for frame in self.frames.borrow().iter() {
            frame.free_objects();
        }

        #[cfg(feature = "dynamic_stack")]
        self.stack.borrow_mut().clear();
        #[cfg(not(feature = "dynamic_stack"))]
        for v in self.stack.borrow_mut().iter_mut() {
            *v = Value::default();
        }

        self.strings.borrow_mut().clear();

        self.globals.borrow_mut().clear();

        // TODO: we need a hash set or something for this to actually work
        // I'm pretty sure the object list itself is over-holding references
        // TODO: this just does not work for leak checking
        // because it runs as things drop (so it prints on *every* drop)
        /*#[cfg(feature = "gc_leak_check")]
        {
            tracing::info!("checking for leaked objects");
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
        }*/

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
    ///
    /// # Panics
    ///
    /// This will panic if the Object is not a Function Object
    pub fn interpret(&self, func: Rc<Object>) -> Result<(), InterpretError> {
        debug!("Interpreting function: {}", func.as_function().get_name());

        self.push(func.clone().into());

        {
            let fp = *self.frame_count.borrow();
            let frame = &mut self.frames.borrow_mut()[fp];
            frame.func = Some(func.as_function().clone());
            frame.ip = 0;
            frame.bp = 0;

            *self.frame_count.borrow_mut() += 1;
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
    fn read_byte<'a>(&self, chunk: &'a Chunk, ip: &mut usize) -> &'a OpCode {
        let ret = chunk.read(*ip);
        *ip += 1;

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
        debug!("Running ...");

        let fp = *self.frame_count.borrow() - 1;
        let frame = &mut self.frames.borrow_mut()[fp];

        if let Some(func) = frame.func.as_ref() {
            let chunk = func.get_chunk().borrow();
            loop {
                let instruction = self.read_byte(&chunk, &mut frame.ip);

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

                    instruction.disassemble("", &chunk, frame.ip - 1);
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
                    OpCode::GetLocal(idx) => {
                        let v = self.stack.borrow()[frame.bp + *idx as usize].clone();
                        self.push(v);
                    }
                    OpCode::SetLocal(idx) => {
                        self.stack.borrow_mut()[frame.bp + *idx as usize] = self.peek(0)
                    }
                    OpCode::GetGlobal(idx) => {
                        // look up the variable name
                        let (name, hash) = self.read_string(&chunk, *idx);

                        if let Some(value) = self.globals.borrow().get(&hash) {
                            // copy the value to the stack
                            self.push(value.clone());
                        } else {
                            self.runtime_error(frame, format!("Undefined variable '{}'.", name));
                            return Err(InterpretError::Runtime("Undefined variable"));
                        }
                    }
                    OpCode::SetGlobal(idx) => {
                        // look up the variable name
                        let (name, hash) = self.read_string(&chunk, *idx);

                        // variable values are not popped off the stack
                        // since assignment is an expression
                        if self
                            .globals
                            .borrow_mut()
                            .insert(hash, self.peek(0))
                            .is_none()
                        {
                            // if the variable didn't exist then it's undefined
                            self.globals.borrow_mut().remove(&hash);
                            self.runtime_error(frame, format!("Undefined variable '{}'.", name));
                            return Err(InterpretError::Runtime("Undefined variable"));
                        }
                    }
                    OpCode::Nil => self.push(Value::Nil),
                    OpCode::False => self.push(false.into()),
                    OpCode::True => self.push(true.into()),
                    OpCode::Equal => self.binary_op(|a, b| Ok(a.equals(b)))?,
                    #[cfg(feature = "extended_opcodes")]
                    OpCode::NotEqual => self.binary_op(|a, b| Ok(a.not_equals(b)))?,

                    // TODO: all of these no longer take self, but pass back the runtime error message in the Err
                    // so we have to pull that out, call runtime_error() by hand, and then re-throw the error
                    OpCode::Less => self.binary_op(|a, b| a.less(b))?,
                    #[cfg(feature = "extended_opcodes")]
                    OpCode::LessEqual => self.binary_op(|a, b| a.less_equal(b, self))?,
                    OpCode::Greater => self.binary_op(|a, b| a.greater(b))?,
                    #[cfg(feature = "extended_opcodes")]
                    OpCode::GreaterEqual => self.binary_op(|a, b| a.greater_equal(b))?,
                    OpCode::Add => self.binary_op(|a, b| a.add(b, self))?,
                    OpCode::Subtract => self.binary_op(|a, b| a.subtract(b))?,
                    OpCode::Multiply => self.binary_op(|a, b| a.multiply(b))?,
                    OpCode::Divide => self.binary_op(|a, b| a.divide(b))?,
                    OpCode::Negate => {
                        let v = self.peek(0).negate()?;

                        self.pop();
                        self.push(v);
                    }
                    OpCode::Not => self.push(self.pop().is_falsey().into()),
                    OpCode::Pop => {
                        self.pop();
                    }
                    OpCode::Print => println!("{}", self.pop()),
                    OpCode::Jump(offset) => frame.ip += *offset as usize - 1,
                    OpCode::JumpIfFalse(offset) => {
                        if self.peek(0).is_falsey() {
                            frame.ip += *offset as usize - 1;
                        }
                    }
                    OpCode::Loop(offset) => frame.ip -= *offset as usize + 1,
                    OpCode::Return => return Ok(()),
                }
            }
        }

        // TODO: should this actually be an error?
        Ok(())
    }

    pub fn runtime_error(&self, frame: &CallFrame, message: impl AsRef<str>) {
        error!("{}", message.as_ref());
        error!(
            "[line {}] in script\n",
            frame
                .func
                .as_ref()
                .unwrap()
                .get_chunk()
                .borrow()
                .get_line(frame.ip - 1)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpret_basic() {
        let vm = VM::new();
        let mut chunk = Chunk::default();
        chunk.write(OpCode::Nil, 123);
        chunk.write(OpCode::Return, 123);
        assert_eq!(
            vm.interpret(Object::from_chunk("main", 0, chunk, &vm))
                .unwrap(),
            ()
        );
    }
}
