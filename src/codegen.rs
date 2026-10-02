//! LLVM IR lowering and the v0.1 Windows native toolchain bridge.

mod llvm;

pub use llvm::{compile_native, generate_llvm_ir};
