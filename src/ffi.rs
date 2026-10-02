//! The deliberately small foreign-function boundary for Princi v0.1.
//!
//! This models C ABI types independently from language types. Managed values,
//! aggregates, pointers, callbacks, and non-C calling conventions do not cross
//! this boundary.

use crate::types::{FunctionType, Type};

/// Primitive C ABI types accepted by Princi's restricted `extern "C"` surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CAbiType {
    Int32,
    Int64,
    Float64,
    Void,
}

impl CAbiType {
    pub fn princi_type(self) -> Type {
        match self {
            Self::Int32 | Self::Int64 => Type::Int,
            Self::Float64 => Type::Float,
            Self::Void => Type::Void,
        }
    }

    pub fn llvm_type(self) -> &'static str {
        match self {
            Self::Int32 => "i32",
            Self::Int64 => "i64",
            Self::Float64 => "double",
            Self::Void => "void",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalFunctionType {
    pub parameters: Vec<CAbiType>,
    pub return_type: CAbiType,
}

impl ExternalFunctionType {
    pub fn new(parameters: Vec<CAbiType>, return_type: CAbiType) -> Self {
        Self {
            parameters,
            return_type,
        }
    }

    /// Princi expressions use `Int` and `Float`; the C signature preserves
    /// widths separately for checked conversion at the ABI boundary.
    pub fn princi_signature(&self) -> FunctionType {
        FunctionType::new(
            self.parameters.iter().map(|ty| ty.princi_type()).collect(),
            self.return_type.princi_type(),
        )
    }
}
