//! Value storage

use std::fmt;
use std::rc::Rc;

use crate::object::*;
use crate::vm::*;

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
    /// Gets the value as a string Object
    ///
    /// # Panics
    ///
    /// This will panic if the value is not a string Object
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
    pub fn negate(&self) -> Result<Self, InterpretError> {
        match self {
            Self::Number(v) => Ok((-v).into()),
            _ => Err(InterpretError::Runtime("Operand must be a number.")),
        }
    }

    #[inline]
    fn number_op<C>(&self, b: Self, op: C) -> Result<Self, InterpretError>
    where
        C: FnOnce(f64, f64) -> Result<Self, InterpretError>,
    {
        match self {
            Self::Number(a) => match b {
                Self::Number(b) => Ok(op(*a, b)?),
                _ => Err(InterpretError::Runtime("Operands must be numbers.")),
            },
            _ => Err(InterpretError::Runtime("Operands must be numbers.")),
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
    pub fn less(&self, other: Self) -> Result<Self, InterpretError> {
        self.number_op(other, |a, b| Ok((a < b).into()))
    }

    /// Compare two (number) values - less than or equal
    #[cfg(feature = "extended_opcodes")]
    #[inline]
    pub fn less_equal(&self, other: Self, vm: &VM) -> Result<Self, InterpretError> {
        self.number_op(other, vm, |a, b| Ok((a <= b).into()))
    }

    /// Compare two (number) values - greater than
    #[inline]
    pub fn greater(&self, other: Self) -> Result<Self, InterpretError> {
        self.number_op(other, |a, b| Ok((a > b).into()))
    }

    /// Compare two (number) values - less than or equal
    #[cfg(feature = "extended_opcodes")]
    #[inline]
    pub fn greater_equal(&self, other: Self, vm: &VM) -> Result<Self, InterpretError> {
        self.number_op(other, vm, |a, b| Ok((a >= b).into()))
    }

    /// Add two (number) values, or concatenate strings
    ///
    /// # Panics
    ///
    /// This will panic when comparing non-String Objects
    #[inline]
    pub fn add(self, other: Self, vm: &VM) -> Result<Self, InterpretError> {
        // TODO: this is a mess
        match self {
            #[cfg(feature = "extended_string_concat")]
            Self::Nil => match other {
                Self::Object(b) => match b.as_ref() {
                    Object::String(b, _) => {
                        Ok(Object::from_string(format!("{}{}", self, b), vm).into())
                    }
                    _ => panic!("Invalid Object concat"),
                },
                _ => Err(InterpretError::Runtime(
                    "Operands must be two numbers or two strings.",
                )),
            },
            #[cfg(feature = "extended_string_concat")]
            Self::Bool(a) => match other {
                Self::Object(b) => match b.as_ref() {
                    Object::String(b, _) => {
                        Ok(Object::from_string(format!("{}{}", a, b), vm).into())
                    }
                    _ => panic!("Invalid Object concat"),
                },
                _ => Err(InterpretError::Runtime(
                    "Operands must be two numbers or two strings.",
                )),
            },
            Self::Number(a) => match other {
                Self::Number(b) => Ok((a + b).into()),
                #[cfg(feature = "extended_string_concat")]
                Self::Object(b) => match b.as_ref() {
                    Object::String(b, _) => {
                        Ok(Object::from_string(format!("{}{}", a, b), vm).into())
                    }
                    _ => panic!("Invalid Object concat"),
                },
                _ => Err(InterpretError::Runtime(
                    "Operands must be two numbers or two strings.",
                )),
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
                            _ => panic!("Invalid Object concat"),
                        },
                        _ => Err(InterpretError::Runtime(
                            "Operands must be two numbers or two strings.",
                        )),
                    }
                }
                _ => panic!("Invalid Object concat"),
            },
            #[cfg(not(feature = "extended_string_concat"))]
            _ => Err(InterpretError::Runtime(
                "Operands must be two numbers or two strings.",
            )),
        }
    }

    /// Subtract two (number) values
    #[inline]
    pub fn subtract(self, other: Self) -> Result<Self, InterpretError> {
        self.number_op(other, |a, b| Ok((a - b).into()))
    }

    /// Multiply two (number) values
    #[inline]
    pub fn multiply(self, other: Self) -> Result<Self, InterpretError> {
        self.number_op(other, |a, b| Ok((a * b).into()))
    }

    /// Divide two (number) values
    #[inline]
    pub fn divide(self, other: Self) -> Result<Self, InterpretError> {
        self.number_op(other, |a, b| {
            #[cfg(not(feature = "allow_divide_by_zero"))]
            if b == 0.0 {
                return Err(InterpretError::Runtime("Illegal divide by zero."));
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
