//! Value storage

use std::fmt;

use crate::vm::*;

/// An heap allocated value
#[derive(Debug, Clone)]
pub enum Object {
    String(String),
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::String(v) => v.fmt(f),
        }
    }
}

// TODO: remove this
impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::String(a) => match other {
                Self::String(b) => a.eq(b),
            },
        }
    }
}

impl From<String> for Object {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<&str> for Object {
    fn from(v: &str) -> Self {
        v.to_owned().into()
    }
}

/// A Value
#[derive(Debug, Default, Clone)]
pub enum Value {
    #[default]
    Nil,
    Bool(bool),
    Number(f64),
    // TODO: would it be possible to support "constant" strings
    // by storing a reference to their slice of the source?
    Object(Object),
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

// TODO: remove this
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Nil => matches!(other, Self::Nil),
            Self::Bool(a) => match other {
                Self::Bool(b) => a.eq(b),
                _ => false,
            },
            Self::Number(a) => match other {
                Self::Number(b) => a.eq(b),
                _ => false,
            },
            Self::Object(a) => match other {
                Self::Object(b) => a.eq(b),
                _ => false,
            },
        }
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::Number(v)
    }
}

impl From<Object> for Value {
    fn from(v: Object) -> Self {
        Self::Object(v)
    }
}

impl Value {
    /// Is this value "falsey"
    pub fn is_falsey(&self) -> bool {
        match self {
            Self::Nil => true,
            Self::Bool(v) => !v,
            _ => false,
        }
    }

    /// Negate a (number) value
    pub fn negate(&self, vm: &VM) -> Result<Value, InterpretError> {
        match self {
            Self::Number(v) => Ok((-v).into()),
            _ => {
                vm.runtime_error("Operand must be a number.");
                Err(InterpretError::Runtime)
            }
        }
    }

    #[inline]
    fn number_op<C>(&self, b: Value, vm: &VM, op: C) -> Result<Value, InterpretError>
    where
        C: FnOnce(f64, f64) -> Result<Value, InterpretError>,
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

    /// Compare two (number) values - less than
    pub fn less(&self, other: Value, vm: &VM) -> Result<Value, InterpretError> {
        self.number_op(other, vm, |a, b| Ok((a < b).into()))
    }

    /// Compare two (number) values - less than or equal
    #[cfg(feature = "extended_opcodes")]
    pub fn less_equal(&self, other: Value, vm: &VM) -> Result<Value, InterpretError> {
        self.number_op(other, vm, |a, b| Ok((a <= b).into()))
    }

    /// Compare two (number) values - greater than
    pub fn greater(&self, other: Value, vm: &VM) -> Result<Value, InterpretError> {
        self.number_op(other, vm, |a, b| Ok((a > b).into()))
    }

    /// Compare two (number) values - less than or equal
    #[cfg(feature = "extended_opcodes")]
    pub fn greater_equal(&self, other: Value, vm: &VM) -> Result<Value, InterpretError> {
        self.number_op(other, vm, |a, b| Ok((a >= b).into()))
    }

    /// Add two (number) values, or concatenate strings
    pub fn add(&self, other: Value, vm: &VM) -> Result<Value, InterpretError> {
        // TODO: concatenate strings
        match self {
            Self::Number(a) => match other {
                Self::Number(b) => Ok((a + b).into()),
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

    /// Subtract two (number) values
    pub fn subtract(&self, other: Value, vm: &VM) -> Result<Value, InterpretError> {
        self.number_op(other, vm, |a, b| Ok((a - b).into()))
    }

    /// Multiply two (number) values
    pub fn multiply(&self, other: Value, vm: &VM) -> Result<Value, InterpretError> {
        self.number_op(other, vm, |a, b| Ok((a * b).into()))
    }

    /// Divide two (number) values
    pub fn divide(&self, other: Value, vm: &VM) -> Result<Value, InterpretError> {
        self.number_op(other, vm, |a, b| {
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
