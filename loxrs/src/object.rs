//! Heap allocated objects

#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::Hasher;
use std::rc::Rc;

use crate::chunk::*;
use crate::vm::*;

/// An heap allocated value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    String(Rc<String>, u64),
    Function(Rc<Function>),
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::String(v, _) => v.fmt(f),
            Self::Function(v) => v.fmt(f),
        }
    }
}

// TODO: this just does not work for leak checking
// because it runs as things drop (so it prints on *every* drop)
/*#[cfg(feature = "gc_leak_check")]
impl Drop for Object {
    fn drop(&mut self) {
        match self {
            Self::String(v, _) => {
                let count = Rc::strong_count(v);
                if count > 1 {
                    tracing::warn!("leaking {} string '{}' strong references", count, v);
                }

                let count = Rc::weak_count(v);
                if count > 0 {
                    tracing::warn!("leaking {} string '{}' weak references", count, v);
                }
            }
            Self::Function(v) => {
                let count = Rc::strong_count(v);
                if count > 1 {
                    tracing::warn!("leaking {} function '{}' strong references", count, v.name);
                }

                let count = Rc::weak_count(v);
                if count > 0 {
                    tracing::warn!("leaking {} function '{}' weak references", count, v.name);
                }
            }
        }
    }
}*/

impl Object {
    /// Creates a new String Object from a String
    pub fn from_string(v: impl Into<String>, vm: &VM) -> Rc<Self> {
        let v = v.into();

        let mut hasher = DefaultHasher::new();
        hasher.write(v.as_bytes());
        let hash = hasher.finish();

        if let Some(v) = vm.find_string(hash) {
            let this = Rc::new(Self::String(v, hash));
            vm.add_object(this.clone());
            return this;
        }

        let v = Rc::new(v);
        vm.add_string(hash, v.clone());

        let this = Rc::new(Self::String(v, hash));
        vm.add_object(this.clone());
        this
    }

    /// Creates a new String Object from a string slice
    pub fn from_str(v: impl AsRef<str>, vm: &VM) -> Rc<Self> {
        let v = v.as_ref();

        let mut hasher = DefaultHasher::new();
        hasher.write(v.as_bytes());
        let hash = hasher.finish();

        if let Some(v) = vm.find_string(hash) {
            let this = Rc::new(Self::String(v, hash));
            vm.add_object(this.clone());
            return this;
        }

        let v = Rc::new(v.to_owned());
        vm.add_string(hash, v.clone());

        let this = Rc::new(Self::String(v, hash));
        vm.add_object(this.clone());
        this
    }

    /// Creates a new Function Object
    pub fn function(name: impl AsRef<str>, vm: &VM) -> Rc<Self> {
        let name = Object::from_str(name, vm);

        let this = Rc::new(Self::Function(Rc::new(Function::new(name))));
        vm.add_object(this.clone());
        this
    }

    /// Creates a new Function Object from a chunk
    pub fn from_chunk(name: impl AsRef<str>, arity: usize, chunk: Chunk, vm: &VM) -> Rc<Self> {
        let name = Object::from_str(name, vm);

        let this = Rc::new(Self::Function(Rc::new(Function::from_chunk(
            name, arity, chunk,
        ))));
        vm.add_object(this.clone());
        this
    }

    /// Creates a new script Function Object
    pub fn script(vm: &VM) -> Rc<Self> {
        let name = Object::from_str("", vm);

        let this = Rc::new(Self::Function(Rc::new(Function::new(name))));
        vm.add_object(this.clone());
        this
    }

    /// Gets the Object string value
    ///
    /// # Panics
    ///
    /// This will panic if the Object is not a String Object
    pub fn as_string(&self) -> (&String, u64) {
        match self {
            Self::String(v, hash) => (v, *hash),
            _ => panic!("Invalid Object as String"),
        }
    }

    /// Gets the Object function
    ///
    /// # Panics
    ///
    /// This will panic if the Object is not a Function Object
    pub fn as_function(&self) -> Rc<Function> {
        match self {
            Self::Function(f) => f.clone(),
            _ => panic!("Invalid Object as Function"),
        }
    }

    /// Compare two objects - equal
    ///
    /// # Panics
    ///
    /// This will panic if the Object is not a String Object
    #[inline]
    pub fn equals(&self, other: &Self) -> bool {
        match self {
            Self::String(a, _) => match other {
                Self::String(b, _) => a == b,
                _ => false,
            },
            _ => panic!("Invalid Object equals"),
        }
    }

    /// Compare two objects - not equal
    #[cfg(feature = "extended_opcodes")]
    #[inline]
    pub fn not_equals(&self, other: Self) -> bool {
        match self {
            Self::String(a) => match other {
                Self::String(b) => *a != b,
            },
        }
    }
}

#[derive(Debug)]
pub struct Function {
    arity: usize,
    chunk: RefCell<Chunk>,

    // must be an Object::String
    name: Rc<Object>,
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self.name.as_ref() {
            Object::String(name, _) => name.as_ref(),
            _ => unreachable!(),
        };

        if name.is_empty() {
            write!(f, "<script>")
        } else {
            write!(f, "<fn {}>", name)
        }
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Function {}

impl Function {
    fn new(name: Rc<Object>) -> Self {
        match name.as_ref() {
            Object::String(_, _) => Self {
                arity: 0,
                chunk: RefCell::new(Chunk::default()),
                name,
            },
            _ => panic!("Invalid function name!"),
        }
    }

    fn from_chunk(name: Rc<Object>, arity: usize, chunk: Chunk) -> Self {
        match name.as_ref() {
            Object::String(_, _) => Self {
                arity,
                chunk: RefCell::new(chunk),
                name,
            },
            _ => panic!("Invalid function name!"),
        }
    }

    pub fn get_name(&self) -> &str {
        match self.name.as_ref() {
            Object::String(name, _) => {
                if name.as_ref().is_empty() {
                    "<script>"
                } else {
                    name.as_ref()
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn get_chunk(&self) -> &RefCell<Chunk> {
        &self.chunk
    }
}

#[cfg(test)]
mod tests {
    // TODO:
}
