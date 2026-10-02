//! LLVM runtime ABI for the x86-64 Windows GNU toolchain.
//!
//! Princi strings are NUL-terminated UTF-8 pointers. Literals are backed by
//! module constants and concatenation allocates a new process-lifetime buffer
//! with the MinGW C runtime.

use crate::types::Type;

pub(crate) const LLVM_RUNTIME: &str = r#"declare i32 @printf(ptr, ...)
declare i32 @putchar(i32)
declare i32 @strcmp(ptr, ptr)
declare i64 @strlen(ptr)
declare ptr @malloc(i64)
declare ptr @memcpy(ptr, ptr, i64)
declare ptr @calloc(i64, i64)

@.princi.rt.fmt.int = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@.princi.rt.fmt.float = private unnamed_addr constant [6 x i8] c"%.15g\00", align 1
@.princi.rt.fmt.string = private unnamed_addr constant [3 x i8] c"%s\00", align 1
@.princi.rt.bool.true = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.princi.rt.bool.false = private unnamed_addr constant [6 x i8] c"false\00", align 1

define internal void @princi_rt_print_int(i64 %value) {
entry:
  %format = getelementptr inbounds [5 x i8], ptr @.princi.rt.fmt.int, i64 0, i64 0
  %ignored = call i32 (ptr, ...) @printf(ptr %format, i64 %value)
  ret void
}

define internal void @princi_rt_println_int(i64 %value) {
entry:
  call void @princi_rt_print_int(i64 %value)
  %ignored = call i32 @putchar(i32 10)
  ret void
}

define internal void @princi_rt_print_float(double %value) {
entry:
  %format = getelementptr inbounds [6 x i8], ptr @.princi.rt.fmt.float, i64 0, i64 0
  %ignored = call i32 (ptr, ...) @printf(ptr %format, double %value)
  ret void
}

define internal void @princi_rt_println_float(double %value) {
entry:
  call void @princi_rt_print_float(double %value)
  %ignored = call i32 @putchar(i32 10)
  ret void
}

define internal void @princi_rt_print_bool(i1 %value) {
entry:
  %true = getelementptr inbounds [5 x i8], ptr @.princi.rt.bool.true, i64 0, i64 0
  %false = getelementptr inbounds [6 x i8], ptr @.princi.rt.bool.false, i64 0, i64 0
  %text = select i1 %value, ptr %true, ptr %false
  call void @princi_rt_print_string(ptr %text)
  ret void
}

define internal void @princi_rt_println_bool(i1 %value) {
entry:
  call void @princi_rt_print_bool(i1 %value)
  %ignored = call i32 @putchar(i32 10)
  ret void
}

define internal void @princi_rt_print_string(ptr %value) {
entry:
  %format = getelementptr inbounds [3 x i8], ptr @.princi.rt.fmt.string, i64 0, i64 0
  %ignored = call i32 (ptr, ...) @printf(ptr %format, ptr %value)
  ret void
}

define internal void @princi_rt_println_string(ptr %value) {
entry:
  call void @princi_rt_print_string(ptr %value)
  %ignored = call i32 @putchar(i32 10)
  ret void
}

define internal ptr @princi_rt_string_concat(ptr %left, ptr %right) {
entry:
  %left.length = call i64 @strlen(ptr %left)
  %right.length = call i64 @strlen(ptr %right)
  %length = add i64 %left.length, %right.length
  %allocation.size = add i64 %length, 1
  %result = call ptr @malloc(i64 %allocation.size)
  %left.copy = call ptr @memcpy(ptr %result, ptr %left, i64 %left.length)
  %destination = getelementptr inbounds i8, ptr %result, i64 %left.length
  %right.size = add i64 %right.length, 1
  %right.copy = call ptr @memcpy(ptr %destination, ptr %right, i64 %right.size)
  ret ptr %result
}

define internal i1 @princi_rt_string_equal(ptr %left, ptr %right) {
entry:
  %comparison = call i32 @strcmp(ptr %left, ptr %right)
  %equal = icmp eq i32 %comparison, 0
  ret i1 %equal
}

define internal ptr @princi_rt_alloc_object(i64 %size) {
entry:
  %object = call ptr @calloc(i64 1, i64 %size)
  ret ptr %object
}
"#;

pub(crate) fn print_function(ty: &Type, newline: bool) -> Option<&'static str> {
    match (ty, newline) {
        (Type::Int, false) => Some("princi_rt_print_int"),
        (Type::Float, false) => Some("princi_rt_print_float"),
        (Type::Bool, false) => Some("princi_rt_print_bool"),
        (Type::String, false) => Some("princi_rt_print_string"),
        (Type::Int, true) => Some("princi_rt_println_int"),
        (Type::Float, true) => Some("princi_rt_println_float"),
        (Type::Bool, true) => Some("princi_rt_println_bool"),
        (Type::String, true) => Some("princi_rt_println_string"),
        _ => None,
    }
}

pub(crate) fn string_concat_function() -> &'static str {
    "princi_rt_string_concat"
}

pub(crate) fn string_equal_function() -> &'static str {
    "princi_rt_string_equal"
}

pub(crate) fn object_allocator_function() -> &'static str {
    "princi_rt_alloc_object"
}

/// Adapt Princi main's return value to the MinGW CRT int main() contract.
pub(crate) fn entry_point(main_symbol: &str, returns_int: bool) -> String {
    let body = if returns_int {
        format!(
            "  %princi.exit = call i64 @{main_symbol}()\n  %princi.exit32 = trunc i64 %princi.exit to i32\n  ret i32 %princi.exit32\n"
        )
    } else {
        format!("  call void @{main_symbol}()\n  ret i32 0\n")
    };
    format!("define i32 @main() {{\nentry:\n{body}}}\n")
}
