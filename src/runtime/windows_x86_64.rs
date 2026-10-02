//! LLVM runtime ABI for the x86-64 Windows GNU toolchain.
//!
//! Managed values use a private process-lifetime heap registry. The allocation
//! ABI is isolated here so a future collector can replace it without changing
//! codegen.

use crate::types::Type;

pub(crate) const LLVM_RUNTIME: &str = r#"declare i32 @printf(ptr, ...)
declare i32 @putchar(i32)
declare i32 @strcmp(ptr, ptr)
declare i64 @strlen(ptr)
declare ptr @malloc(i64)
declare void @free(ptr)
declare ptr @memcpy(ptr, ptr, i64)
declare ptr @memset(ptr, i32, i64)
declare void @exit(i32) noreturn

@.princi.rt.fmt.int = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@.princi.rt.fmt.float = private unnamed_addr constant [6 x i8] c"%.15g\00", align 1
@.princi.rt.fmt.string = private unnamed_addr constant [3 x i8] c"%s\00", align 1
@.princi.rt.bool.true = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.princi.rt.bool.false = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.princi.rt.managed.allocation.error = private unnamed_addr constant [27 x i8] c"managed allocation failed\0A\00", align 1
@.princi.rt.ffi.int32.error = private unnamed_addr constant [33 x i8] c"FFI Int32 argument out of range\0A\00", align 1

%princi.rt.managed.block = type { ptr }
@.princi.rt.managed.head = internal global ptr null, align 8

define internal void @princi_rt_managed_shutdown() {
entry:
  br label %loop
loop:
  %current = load ptr, ptr @.princi.rt.managed.head
  %finished = icmp eq ptr %current, null
  br i1 %finished, label %done, label %release
release:
  %next.slot = getelementptr inbounds %princi.rt.managed.block, ptr %current, i32 0, i32 0
  %next = load ptr, ptr %next.slot
  store ptr %next, ptr @.princi.rt.managed.head
  call void @free(ptr %current)
  br label %loop
done:
  ret void
}

define internal void @princi_rt_managed_panic_allocation() {
entry:
  %message = getelementptr inbounds [27 x i8], ptr @.princi.rt.managed.allocation.error, i64 0, i64 0
  %ignored = call i32 (ptr, ...) @printf(ptr %message)
  call void @princi_rt_managed_shutdown()
  call void @exit(i32 1)
  unreachable
}

define internal i32 @princi_rt_ffi_int32_checked(i64 %value) {
entry:
  %above.minimum = icmp sge i64 %value, -2147483648
  %below.maximum = icmp sle i64 %value, 2147483647
  %in.range = and i1 %above.minimum, %below.maximum
  br i1 %in.range, label %convert, label %invalid
invalid:
  %message = getelementptr inbounds [33 x i8], ptr @.princi.rt.ffi.int32.error, i64 0, i64 0
  %ignored = call i32 (ptr, ...) @printf(ptr %message)
  call void @princi_rt_managed_shutdown()
  call void @exit(i32 1)
  unreachable
convert:
  %narrow = trunc i64 %value to i32
  ret i32 %narrow
}

define internal ptr @princi_rt_managed_alloc(i64 %size) {
entry:
  %valid.size = icmp sge i64 %size, 0
  br i1 %valid.size, label %layout, label %invalid
invalid:
  call void @princi_rt_managed_panic_allocation()
  unreachable
layout:
  %header.end = getelementptr %princi.rt.managed.block, ptr null, i32 1
  %header.size = ptrtoint ptr %header.end to i64
  %total.size = add i64 %header.size, %size
  %overflow = icmp ult i64 %total.size, %header.size
  br i1 %overflow, label %invalid, label %allocate
allocate:
  %block = call ptr @malloc(i64 %total.size)
  %allocation.failed = icmp eq ptr %block, null
  br i1 %allocation.failed, label %invalid, label %register
register:
  %next.slot = getelementptr inbounds %princi.rt.managed.block, ptr %block, i32 0, i32 0
  %head = load ptr, ptr @.princi.rt.managed.head
  store ptr %head, ptr %next.slot
  store ptr %block, ptr @.princi.rt.managed.head
  %payload = getelementptr inbounds i8, ptr %block, i64 %header.size
  ret ptr %payload
}

define internal ptr @princi_rt_managed_calloc(i64 %size) {
entry:
  %memory = call ptr @princi_rt_managed_alloc(i64 %size)
  %ignored = call ptr @memset(ptr %memory, i32 0, i64 %size)
  ret ptr %memory
}

define internal ptr @princi_rt_managed_grow(ptr %old, i64 %old.size, i64 %new.size) {
entry:
  ; Keep the old block registered; shutdown reclaims it with every other block.
  %memory = call ptr @princi_rt_managed_alloc(i64 %new.size)
  %empty = icmp eq i64 %old.size, 0
  br i1 %empty, label %done, label %copy
copy:
  %ignored = call ptr @memcpy(ptr %memory, ptr %old, i64 %old.size)
  br label %done
done:
  ret ptr %memory
}

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
  %length.overflow = icmp ult i64 %length, %left.length
  br i1 %length.overflow, label %allocation.failed, label %size
size:
  %allocation.size = add i64 %length, 1
  %size.overflow = icmp ult i64 %allocation.size, %length
  br i1 %size.overflow, label %allocation.failed, label %allocate
allocation.failed:
  call void @princi_rt_managed_panic_allocation()
  unreachable
allocate:
  %result = call ptr @princi_rt_managed_alloc(i64 %allocation.size)
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
  %object = call ptr @princi_rt_managed_calloc(i64 %size)
  ret ptr %object
}
"#;

/// Private byte-copy ABI for homogeneous, heap-backed \`List<T>\` values.
/// The compiler supplies the target layout's element size for every operation.
pub(crate) const LIST_RUNTIME: &str = r#"%princi.rt.list = type { ptr, i64, i64, i64 }
@.princi.rt.list.bounds = private unnamed_addr constant [26 x i8] c"list index out of bounds\0A\00", align 1
@.princi.rt.list.type = private unnamed_addr constant [28 x i8] c"list element type mismatch\0A\00", align 1
@.princi.rt.list.allocation = private unnamed_addr constant [24 x i8] c"list allocation failed\0A\00", align 1
define internal void @princi_rt_list_panic_bounds() {
entry:
  %message = getelementptr inbounds [26 x i8], ptr @.princi.rt.list.bounds, i64 0, i64 0
  %ignored = call i32 (ptr, ...) @printf(ptr %message)
  call void @princi_rt_managed_shutdown()
  call void @exit(i32 1)
  unreachable
}
define internal void @princi_rt_list_panic_type() {
entry:
  %message = getelementptr inbounds [28 x i8], ptr @.princi.rt.list.type, i64 0, i64 0
  %ignored = call i32 (ptr, ...) @printf(ptr %message)
  call void @princi_rt_managed_shutdown()
  call void @exit(i32 1)
  unreachable
}
define internal void @princi_rt_list_panic_allocation() {
entry:
  %message = getelementptr inbounds [24 x i8], ptr @.princi.rt.list.allocation, i64 0, i64 0
  %ignored = call i32 (ptr, ...) @printf(ptr %message)
  call void @princi_rt_managed_shutdown()
  call void @exit(i32 1)
  unreachable
}
define internal ptr @princi_rt_list_new(i64 %element.size) {
entry:
  %valid.size = icmp sge i64 %element.size, 0
  br i1 %valid.size, label %allocate, label %invalid
invalid:
  call void @princi_rt_list_panic_type()
  unreachable
allocate:
  %header.end = getelementptr %princi.rt.list, ptr null, i32 1
  %header.size = ptrtoint ptr %header.end to i64
  %list = call ptr @princi_rt_managed_calloc(i64 %header.size)
  %element.size.slot = getelementptr inbounds %princi.rt.list, ptr %list, i32 0, i32 3
  store i64 %element.size, ptr %element.size.slot
  ret ptr %list
}
define internal void @princi_rt_list_check_type(ptr %list, i64 %expected.size) {
entry:
  %is.null = icmp eq ptr %list, null
  br i1 %is.null, label %invalid, label %compare
invalid:
  call void @princi_rt_list_panic_type()
  unreachable
compare:
  %element.size.slot = getelementptr inbounds %princi.rt.list, ptr %list, i32 0, i32 3
  %element.size = load i64, ptr %element.size.slot
  %matches = icmp eq i64 %element.size, %expected.size
  br i1 %matches, label %done, label %mismatch
mismatch:
  call void @princi_rt_list_panic_type()
  unreachable
done:
  ret void
}
define internal i64 @princi_rt_list_length(ptr %list) {
entry:
  %is.null = icmp eq ptr %list, null
  br i1 %is.null, label %invalid, label %read.length
invalid:
  call void @princi_rt_list_panic_type()
  unreachable
read.length:
  %length.slot = getelementptr inbounds %princi.rt.list, ptr %list, i32 0, i32 1
  %length = load i64, ptr %length.slot
  ret i64 %length
}
define internal ptr @princi_rt_list_slot(ptr %list, i64 %index, i64 %expected.size) {
entry:
  call void @princi_rt_list_check_type(ptr %list, i64 %expected.size)
  %length.slot = getelementptr inbounds %princi.rt.list, ptr %list, i32 0, i32 1
  %length = load i64, ptr %length.slot
  %negative = icmp slt i64 %index, 0
  %past.end = icmp sge i64 %index, %length
  %out.of.bounds = or i1 %negative, %past.end
  br i1 %out.of.bounds, label %bounds.error, label %address
bounds.error:
  call void @princi_rt_list_panic_bounds()
  unreachable
address:
  %data.slot = getelementptr inbounds %princi.rt.list, ptr %list, i32 0, i32 0
  %data = load ptr, ptr %data.slot
  %zero.size = icmp eq i64 %expected.size, 0
  %storage.size = select i1 %zero.size, i64 1, i64 %expected.size
  %offset = mul i64 %index, %storage.size
  %element = getelementptr inbounds i8, ptr %data, i64 %offset
  ret ptr %element
}
define internal void @princi_rt_list_get(ptr %list, i64 %index, i64 %element.size, ptr %destination) {
entry:
  %element = call ptr @princi_rt_list_slot(ptr %list, i64 %index, i64 %element.size)
  %copied = call ptr @memcpy(ptr %destination, ptr %element, i64 %element.size)
  ret void
}
define internal void @princi_rt_list_set(ptr %list, i64 %index, i64 %element.size, ptr %source) {
entry:
  %element = call ptr @princi_rt_list_slot(ptr %list, i64 %index, i64 %element.size)
  %copied = call ptr @memcpy(ptr %element, ptr %source, i64 %element.size)
  ret void
}
define internal void @princi_rt_list_add(ptr %list, i64 %element.size, ptr %source) {
entry:
  call void @princi_rt_list_check_type(ptr %list, i64 %element.size)
  %length.slot = getelementptr inbounds %princi.rt.list, ptr %list, i32 0, i32 1
  %capacity.slot = getelementptr inbounds %princi.rt.list, ptr %list, i32 0, i32 2
  %data.slot = getelementptr inbounds %princi.rt.list, ptr %list, i32 0, i32 0
  %length = load i64, ptr %length.slot
  %capacity = load i64, ptr %capacity.slot
  %old.data = load ptr, ptr %data.slot
  %zero.size = icmp eq i64 %element.size, 0
  %storage.size = select i1 %zero.size, i64 1, i64 %element.size
  %full = icmp eq i64 %length, %capacity
  br i1 %full, label %grow.start, label %append.existing
append.existing:
  br label %append
grow.start:
  %empty = icmp eq i64 %capacity, 0
  br i1 %empty, label %grow.initial, label %grow.check
grow.initial:
  br label %grow.capacity
grow.check:
  %can.double = icmp ule i64 %capacity, 4611686018427387903
  br i1 %can.double, label %grow.double, label %growth.error
grow.double:
  %doubled = mul i64 %capacity, 2
  br label %grow.capacity
grow.capacity:
  %new.capacity = phi i64 [ 4, %grow.initial ], [ %doubled, %grow.double ]
  %max.capacity = sdiv i64 9223372036854775807, %storage.size
  %capacity.fits = icmp sle i64 %new.capacity, %max.capacity
  br i1 %capacity.fits, label %allocate.data, label %growth.error
allocate.data:
  %new.bytes = mul i64 %new.capacity, %storage.size
  %old.bytes = mul i64 %capacity, %storage.size
  %new.data = call ptr @princi_rt_managed_grow(ptr %old.data, i64 %old.bytes, i64 %new.bytes)
  br label %grow.done
growth.error:
  call void @princi_rt_list_panic_allocation()
  unreachable
grow.done:
  store ptr %new.data, ptr %data.slot
  store i64 %new.capacity, ptr %capacity.slot
  br label %append
append:
  %data = phi ptr [ %old.data, %append.existing ], [ %new.data, %grow.done ]
  %offset = mul i64 %length, %storage.size
  %destination = getelementptr inbounds i8, ptr %data, i64 %offset
  %copied = call ptr @memcpy(ptr %destination, ptr %source, i64 %element.size)
  %new.length = add i64 %length, 1
  store i64 %new.length, ptr %length.slot
  ret void
}"#;
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

pub(crate) fn list_new_function() -> &'static str {
    "princi_rt_list_new"
}

pub(crate) fn list_length_function() -> &'static str {
    "princi_rt_list_length"
}

pub(crate) fn list_get_function() -> &'static str {
    "princi_rt_list_get"
}

pub(crate) fn list_set_function() -> &'static str {
    "princi_rt_list_set"
}

pub(crate) fn list_add_function() -> &'static str {
    "princi_rt_list_add"
}

/// Adapt Princi main's return value to the MinGW CRT int main() contract.
pub(crate) fn entry_point(main_symbol: &str, returns_int: bool) -> String {
    let body = if returns_int {
        format!(
            "  %princi.exit = call i64 @{main_symbol}()\n  call void @princi_rt_managed_shutdown()\n  %princi.exit32 = trunc i64 %princi.exit to i32\n  ret i32 %princi.exit32\n"
        )
    } else {
        format!(
            "  call void @{main_symbol}()\n  call void @princi_rt_managed_shutdown()\n  ret i32 0\n"
        )
    };
    format!("define i32 @main() {{\nentry:\n{body}}}\n")
}
