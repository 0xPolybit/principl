/// Small LLVM runtime used by the v0.1 procedural native backend.
///
/// Princi strings use NUL-terminated UTF-8 byte storage at the Windows C ABI
/// boundary. The helper implementations use the MinGW C runtime.
pub(crate) const LLVM_RUNTIME: &str = r#"declare i32 @printf(ptr, ...)
declare i32 @puts(ptr)
declare i32 @strcmp(ptr, ptr)
declare i64 @strlen(ptr)
declare ptr @malloc(i64)
declare ptr @memcpy(ptr, ptr, i64)

@.princi.fmt.int = private unnamed_addr constant [6 x i8] c"%lld\0A\00", align 1
@.princi.fmt.float = private unnamed_addr constant [7 x i8] c"%.15g\0A\00", align 1
@.princi.fmt.string = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@.princi.bool.true = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.princi.bool.false = private unnamed_addr constant [6 x i8] c"false\00", align 1

define internal void @princi_print_int(i64 %value) {
entry:
  %format = getelementptr inbounds [6 x i8], ptr @.princi.fmt.int, i64 0, i64 0
  %ignored = call i32 (ptr, ...) @printf(ptr %format, i64 %value)
  ret void
}

define internal void @princi_print_float(double %value) {
entry:
  %format = getelementptr inbounds [7 x i8], ptr @.princi.fmt.float, i64 0, i64 0
  %ignored = call i32 (ptr, ...) @printf(ptr %format, double %value)
  ret void
}

define internal void @princi_print_bool(i1 %value) {
entry:
  %true = getelementptr inbounds [5 x i8], ptr @.princi.bool.true, i64 0, i64 0
  %false = getelementptr inbounds [6 x i8], ptr @.princi.bool.false, i64 0, i64 0
  %text = select i1 %value, ptr %true, ptr %false
  %ignored = call i32 @puts(ptr %text)
  ret void
}

define internal void @princi_print_string(ptr %value) {
entry:
  %format = getelementptr inbounds [4 x i8], ptr @.princi.fmt.string, i64 0, i64 0
  %ignored = call i32 (ptr, ...) @printf(ptr %format, ptr %value)
  ret void
}

define internal ptr @princi_concat(ptr %left, ptr %right) {
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
"#;
