//! LLVM IR lowering and the Windows x86-64 native toolchain bridge.

mod llvm;

pub use llvm::{compile_native, generate_llvm_ir};
