//! Value storage

use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::Hasher;
use std::rc::Rc;

use crate::vm::*;

/// An heap allocated value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    String(Rc<String>, u64),
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::String(v, _) => v.fmt(f),
        }
    }
}

#[cfg(feature = "gc_leak_check")]
impl Drop for Object {
    fn drop(&mut self) {
        match self {
            Self::String(v, _) => {
                let count = Rc::strong_count(v);
                if count > 1 {
                    tracing::warn!("leaking {} string strong references", count);
                }

                let count = Rc::weak_count(v);
                if count > 0 {
                    tracing::warn!("leaking {} string weak references", count);
                }
            }
        }
    }
}

impl Object {
    /// Creates a new string Object from a String
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

    /// Creates a new string Object from a string slice
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

    /// Gets the Object string value
    ///
    /// This will panic if the object is not a string object
    pub fn as_string(&self) -> (&String, u64) {
        match self {
            Self::String(v, hash) => (v, *hash),
        }
    }

    /// Compare two objects - equal
    #[inline]
    pub fn equals(&self, other: &Self) -> bool {
        match self {
            Self::String(a, _) => match other {
                Self::String(b, _) => a == b,
            },
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

/// A Value
#[derive(Debug, Default, Clone, PartialEq)]
pub enum Value {
    #[default]
    Nil,
    Bool(bool),
    Number(f64),
    // TODO: would it be possible to support "constant" strings
    // by storing a reference to their slice of the source?
    Object(Rc<Object>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(v) => v.fmt(f),
            Self::Number(v) => v.fmt(f),
            Self::Object(v) => v.fmt(f),
        }
    }
}

impl From<bool> for Value {
    #[inline]
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<f64> for Value {
    #[inline]
    fn from(v: f64) -> Self {
        Self::Number(v)
    }
}

impl From<Rc<Object>> for Value {
    #[inline]
    fn from(v: Rc<Object>) -> Self {
        Self::Object(v)
    }
}

impl Value {
    /// Gets the value as a string object
    ///
    /// This will panic if the value is not a string object
    pub fn as_string(&self) -> (&String, u64) {
        match self {
            Self::Object(v) => v.as_string(),
            _ => panic!("Invalid Value to String"),
        }
    }

    /// Is this value "falsey"
    #[inline]
    pub fn is_falsey(&self) -> bool {
        match self {
            Self::Nil => true,
            Self::Bool(v) => !v,
            _ => false,
        }
    }

    /// Negate a (number) value
    #[inline]
    pub fn negate(&self, vm: &VM) -> Result<Self, InterpretError> {
        match self {
            Self::Number(v) => Ok((-v).into()),
            _ => {
                vm.runtime_error("Operand must be a number.");
                Err(InterpretError::Runtime)
            }
        }
    }

    #[inline]
    fn number_op<C>(&self, b: Self, vm: &VM, op: C) -> Result<Self, InterpretError>
    where
        C: FnOnce(f64, f64) -> Result<Self, InterpretError>,
    {
        match self {
            Self::Number(a) => match b {
                Self::Number(b) => Ok(op(*a, b)?),
                _ => {
                    vm.runtime_error("Operands must be numbers.");
                    Err(InterpretError::Runtime)
                }
            },
            _ => {
                vm.runtime_error("Operands must be numbers.");
                Err(InterpretError::Runtime)
            }
        }
    }

    /// Compare two values - equal
    #[inline]
    pub fn equals(&self, other: Self) -> Self {
        match self {
            Self::Nil => matches!(other, Self::Nil),
            Self::Bool(a) => match other {
                Self::Bool(b) => *a == b,
                _ => false,
            },
            Self::Number(a) => match other {
                Self::Number(b) => *a == b,
                _ => false,
            },
            Self::Object(a) => match other {
                Self::Object(b) => a.equals(b.as_ref()),
                _ => false,
            },
        }
        .into()
    }

    /// Compare two values - not equal
    #[cfg(feature = "extended_opcodes")]
    #[inline]
    pub fn not_equals(&self, other: Self) -> Self {
        match self {
            Self::Nil => matches!(other, Self::Nil),
            Self::Bool(a) => match other {
                Self::Bool(b) => *a != b,
                _ => false,
            },
            Self::Number(a) => match other {
                Self::Number(b) => *a != b,
                _ => false,
            },
            Self::Object(a) => match other {
                Self::Object(b) => *a != b,
                _ => false,
            },
        }
        .into()
    }

    /// Compare two (number) values - less than
    #[inline]
    pub fn less(&self, other: Self, vm: &VM) -> Result<Self, InterpretError> {
        self.number_op(other, vm, |a, b| Ok((a < b).into()))
    }

    /// Compare two (number) values - less than or equal
    #[cfg(feature = "extended_opcodes")]
    #[inline]
    pub fn less_equal(&self, other: Self, vm: &VM) -> Result<Self, InterpretError> {
        self.number_op(other, vm, |a, b| Ok((a <= b).into()))
    }

    /// Compare two (number) values - greater than
    #[inline]
    pub fn greater(&self, other: Self, vm: &VM) -> Result<Self, InterpretError> {
        self.number_op(other, vm, |a, b| Ok((a > b).into()))
    }

    /// Compare two (number) values - less than or equal
    #[cfg(feature = "extended_opcodes")]
    #[inline]
    pub fn greater_equal(&self, other: Self, vm: &VM) -> Result<Self, InterpretError> {
        self.number_op(other, vm, |a, b| Ok((a >= b).into()))
    }

    /// Add two (number) values, or concatenate strings
    #[inline]
    pub fn add(self, other: Self, vm: &VM) -> Result<Self, InterpretError> {
        match self {
            #[cfg(feature = "extended_string_concat")]
            Self::Nil => match other {
                Self::Object(b) => match b.as_ref() {
                    Object::String(b, _) => {
                        Ok(Object::from_string(format!("{}{}", self, b), vm).into())
                    }
                },
                _ => {
                    vm.runtime_error("Operands must be two numbers or two strings.");
                    Err(InterpretError::Runtime)
                }
            },
            #[cfg(feature = "extended_string_concat")]
            Self::Bool(a) => match other {
                Self::Object(b) => match b.as_ref() {
                    Object::String(b, _) => {
                        Ok(Object::from_string(format!("{}{}", a, b), vm).into())
                    }
                },
                _ => {
                    vm.runtime_error("Operands must be two numbers or two strings.");
                    Err(InterpretError::Runtime)
                }
            },
            Self::Number(a) => match other {
                Self::Number(b) => Ok((a + b).into()),
                #[cfg(feature = "extended_string_concat")]
                Self::Object(b) => match b.as_ref() {
                    Object::String(b, _) => {
                        Ok(Object::from_string(format!("{}{}", a, b), vm).into())
                    }
                },
                _ => {
                    vm.runtime_error("Operands must be two numbers or two strings.");
                    Err(InterpretError::Runtime)
                }
            },
            Self::Object(a) => match a.as_ref() {
                Object::String(a, _) => {
                    #[cfg(feature = "extended_string_concat")]
                    {
                        Ok(Object::from_string(format!("{}{}", a, other), vm).into())
                    }

                    #[cfg(not(feature = "extended_string_concat"))]
                    match other {
                        Self::Object(b) => match b.as_ref() {
                            Object::String(b, _) => {
                                Ok(Object::from_string(format!("{}{}", a, b), vm).into())
                            }
                        },
                        _ => {
                            vm.runtime_error("Operands must be two numbers or two strings.");
                            Err(InterpretError::Runtime)
                        }
                    }
                }
            },
            #[cfg(not(feature = "extended_string_concat"))]
            _ => {
                vm.runtime_error("Operands must be two numbers or two strings.");
                Err(InterpretError::Runtime)
            }
        }
    }

    /// Subtract two (number) values
    #[inline]
    pub fn subtract(self, other: Self, vm: &VM) -> Result<Self, InterpretError> {
        self.number_op(other, vm, |a, b| Ok((a - b).into()))
    }

    /// Multiply two (number) values
    #[inline]
    pub fn multiply(self, other: Self, vm: &VM) -> Result<Self, InterpretError> {
        self.number_op(other, vm, |a, b| Ok((a * b).into()))
    }

    /// Divide two (number) values
    #[inline]
    pub fn divide(self, other: Self, vm: &VM) -> Result<Self, InterpretError> {
        self.number_op(other, vm, |a, b| {
            #[cfg(not(feature = "allow_divide_by_zero"))]
            if b == 0.0 {
                vm.runtime_error("Illegal divide by zero.");
                return Err(InterpretError::Runtime);
            }
            Ok((a / b).into())
        })
    }
}

/// A set of Values
pub type ValueArray = Vec<Value>;

#[cfg(test)]
mod tests {
    // TODO:
}
