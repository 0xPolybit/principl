//! Runtime ABI and platform implementations used by generated code.
//!
//! Language-level built-ins are lowered through the platform runtime module;
//! implementation symbols are compiler-private and are never source names.

pub(crate) mod windows_x86_64;
