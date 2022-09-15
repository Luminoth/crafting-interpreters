//! Value storage

use std::fmt;

use crate::vm::*;

/// A Value
#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub enum Value {
    #[default]
    Nil,
    Bool(bool),
    Number(f64),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(v) => write!(f, "{}", v),
            Self::Number(v) => write!(f, "{}", v),
        }
    }
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Self::Nil => true,
            Self::Bool(v) => !v,
            _ => false,
        }
    }

    pub fn negate(&self, vm: &VM) -> Result<Value, InterpretError> {
        match self {
            Self::Number(v) => Ok(Value::Number(-v)),
            _ => {
                vm.runtime_error("Operand must be a number.");
                Err(InterpretError::Runtime)
            }
        }
    }

    #[inline]
    fn number_op<C>(&self, b: Value, vm: &VM, op: C) -> Result<Value, InterpretError>
    where
        C: FnOnce(f64, f64) -> Result<f64, InterpretError>,
    {
        match self {
            Self::Number(a) => match b {
                Self::Number(b) => Ok(Value::Number(op(*a, b)?)),
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

    pub fn add(&self, other: Value, vm: &VM) -> Result<Value, InterpretError> {
        // TODO: concatenate strings
        match self {
            Self::Number(a) => match other {
                Self::Number(b) => Ok(Value::Number(a + b)),
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

    pub fn subtract(&self, other: Value, vm: &VM) -> Result<Value, InterpretError> {
        self.number_op(other, vm, |a, b| Ok(a - b))
    }

    pub fn multiply(&self, other: Value, vm: &VM) -> Result<Value, InterpretError> {
        self.number_op(other, vm, |a, b| Ok(a * b))
    }

    pub fn divide(&self, other: Value, vm: &VM) -> Result<Value, InterpretError> {
        self.number_op(other, vm, |a, b| {
            if b == 0.0 {
                vm.runtime_error("Illegal divide by zero.");
                return Err(InterpretError::Runtime);
            }
            Ok(a / b)
        })
    }
}

/// A set of Values
pub type ValueArray = Vec<Value>;
