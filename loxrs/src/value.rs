//! Value storage

use std::fmt;

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
}

/// A set of Values
pub type ValueArray = Vec<Value>;
