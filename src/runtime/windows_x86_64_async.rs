//! Private cooperative async runtime ABI for Windows x86-64.
//!
//! Tasks are opaque managed handles. Their bodies execute on Windows fibers,
//! allowing the compiler to preserve ordinary LLVM control flow while await
//! yields back to a deterministic single-thread scheduler.

pub(crate) const ASYNC_RUNTIME: &str = r#"
declare ptr @ConvertThreadToFiber(ptr)
declare ptr @CreateFiber(i64, ptr, ptr)
declare void @SwitchToFiber(ptr)
declare void @DeleteFiber(ptr)
declare i32 @ConvertFiberToThread()

%princi.rt.task = type { i32, ptr, ptr, ptr, ptr, i64, ptr, ptr, ptr, ptr, ptr }
%princi.rt.scope = type { ptr, ptr }
@.princi.rt.async.deadlock = private unnamed_addr constant [37 x i8] c"async runtime has no runnable tasks\0A\00", align 1
@.princi.rt.async.fiber = private unnamed_addr constant [43 x i8] c"async runtime could not create task fiber\0A\00", align 1
@.princi.rt.async.scheduler = internal global ptr null, align 8
@.princi.rt.async.current = internal global ptr null, align 8
@.princi.rt.async.tasks = internal global ptr null, align 8
@.princi.rt.async.ready.head = internal global ptr null, align 8
@.princi.rt.async.ready.tail = internal global ptr null, align 8

; State values: created=0, ready=1, running=2, pending=3, completed=4.
define internal void @princi_rt_task_panic(ptr %message) {
entry:
  %ignored = call i32 (ptr, ...) @printf(ptr %message)
  call void @exit(i32 1)
  unreachable
}

define internal void @princi_rt_task_runtime_init() {
entry:
  %existing = load ptr, ptr @.princi.rt.async.scheduler
  %is.ready = icmp ne ptr %existing, null
  br i1 %is.ready, label %done, label %convert
convert:
  %fiber = call ptr @ConvertThreadToFiber(ptr null)
  %failed = icmp eq ptr %fiber, null
  br i1 %failed, label %invalid, label %store
invalid:
  %message = getelementptr inbounds [43 x i8], ptr @.princi.rt.async.fiber, i64 0, i64 0
  call void @princi_rt_task_panic(ptr %message)
  unreachable
store:
  store ptr %fiber, ptr @.princi.rt.async.scheduler
  br label %done
done:
  ret void
}

define internal ptr @princi_rt_task_create(ptr %callback, i64 %result.size, ptr %arguments, i64 %arguments.size) {
entry:
  %task.end = getelementptr %princi.rt.task, ptr null, i32 1
  %task.size = ptrtoint ptr %task.end to i64
  %task = call ptr @princi_rt_managed_calloc(i64 %task.size)
  %result = call ptr @princi_rt_managed_calloc(i64 %result.size)
  %context = call ptr @princi_rt_managed_alloc(i64 %arguments.size)
  %context.copy = call ptr @memcpy(ptr %context, ptr %arguments, i64 %arguments.size)
  %callback.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 2
  store ptr %callback, ptr %callback.slot
  %context.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 3
  store ptr %context, ptr %context.slot
  %result.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 4
  store ptr %result, ptr %result.slot
  %result.size.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 5
  store i64 %result.size, ptr %result.size.slot
  %all.next.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 6
  %head = load ptr, ptr @.princi.rt.async.tasks
  store ptr %head, ptr %all.next.slot
  store ptr %task, ptr @.princi.rt.async.tasks
  ret ptr %task
}

define internal void @princi_rt_task_enqueue(ptr %task) {
entry:
  %state.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 0
  %state = load i32, ptr %state.slot
  %created = icmp eq i32 %state, 0
  %pending = icmp eq i32 %state, 3
  %can.enqueue = or i1 %created, %pending
  br i1 %can.enqueue, label %prepare, label %done
prepare:
  %fiber.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 1
  %fiber = load ptr, ptr %fiber.slot
  %needs.fiber = icmp eq ptr %fiber, null
  br i1 %needs.fiber, label %create, label %queue
create:
  %new.fiber = call ptr @CreateFiber(i64 0, ptr @princi_rt_task_start, ptr %task)
  %failed = icmp eq ptr %new.fiber, null
  br i1 %failed, label %invalid, label %store.fiber
invalid:
  %message = getelementptr inbounds [43 x i8], ptr @.princi.rt.async.fiber, i64 0, i64 0
  call void @princi_rt_task_panic(ptr %message)
  unreachable
store.fiber:
  store ptr %new.fiber, ptr %fiber.slot
  br label %queue
queue:
  store i32 1, ptr %state.slot
  %next.ready.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 7
  store ptr null, ptr %next.ready.slot
  %tail = load ptr, ptr @.princi.rt.async.ready.tail
  %empty = icmp eq ptr %tail, null
  br i1 %empty, label %first, label %append
first:
  store ptr %task, ptr @.princi.rt.async.ready.head
  store ptr %task, ptr @.princi.rt.async.ready.tail
  br label %done
append:
  %tail.next.slot = getelementptr inbounds %princi.rt.task, ptr %tail, i32 0, i32 7
  store ptr %task, ptr %tail.next.slot
  store ptr %task, ptr @.princi.rt.async.ready.tail
  br label %done
done:
  ret void
}

define internal ptr @princi_rt_task_dequeue() {
entry:
  %task = load ptr, ptr @.princi.rt.async.ready.head
  %empty = icmp eq ptr %task, null
  br i1 %empty, label %done, label %remove
remove:
  %next.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 7
  %next = load ptr, ptr %next.slot
  store ptr %next, ptr @.princi.rt.async.ready.head
  %last = icmp eq ptr %next, null
  br i1 %last, label %clear.tail, label %finish
clear.tail:
  store ptr null, ptr @.princi.rt.async.ready.tail
  br label %finish
finish:
  store ptr null, ptr %next.slot
  ret ptr %task
done:
  ret ptr null
}


define internal void @princi_rt_task_wake_waiters(ptr %completed) {
entry:
  %first = load ptr, ptr @.princi.rt.async.tasks
  br label %loop
loop:
  %task = phi ptr [ %first, %entry ], [ %next, %continue ]
  %finished = icmp eq ptr %task, null
  br i1 %finished, label %done, label %check
check:
  %state.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 0
  %state = load i32, ptr %state.slot
  %is.pending = icmp eq i32 %state, 3
  %waiting.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 8
  %waiting = load ptr, ptr %waiting.slot
  %matches = icmp eq ptr %waiting, %completed
  %wake = and i1 %is.pending, %matches
  br i1 %wake, label %enqueue, label %continue
enqueue:
  store ptr null, ptr %waiting.slot
  call void @princi_rt_task_enqueue(ptr %task)
  br label %continue
continue:
  %next.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 6
  %next = load ptr, ptr %next.slot
  br label %loop
done:
  ret void
}

define internal void @princi_rt_task_start(ptr %task) {
entry:
  store ptr %task, ptr @.princi.rt.async.current
  %root.scope = alloca %princi.rt.scope, align 8
  call void @princi_rt_scope_enter(ptr %root.scope)
  %callback.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 2
  %callback = load ptr, ptr %callback.slot
  %context.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 3
  %context = load ptr, ptr %context.slot
  call void %callback(ptr %task, ptr %context)
  call void @princi_rt_scope_exit(ptr %root.scope)
  %state.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 0
  store i32 4, ptr %state.slot
  store ptr null, ptr @.princi.rt.async.current
  %scheduler = load ptr, ptr @.princi.rt.async.scheduler
  call void @SwitchToFiber(ptr %scheduler)
  unreachable
}

define internal void @princi_rt_task_spawn(ptr %task) {
entry:
  %current = load ptr, ptr @.princi.rt.async.current
  %invalid = icmp eq ptr %current, null
  br i1 %invalid, label %panic, label %attach
panic:
  %message = getelementptr inbounds [37 x i8], ptr @.princi.rt.async.deadlock, i64 0, i64 0
  call void @princi_rt_task_panic(ptr %message)
  unreachable
attach:
  %scope.slot = getelementptr inbounds %princi.rt.task, ptr %current, i32 0, i32 9
  %scope = load ptr, ptr %scope.slot
  %no.scope = icmp eq ptr %scope, null
  br i1 %no.scope, label %panic, label %check.child
check.child:
  %child.scope.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 9
  %child.scope = load ptr, ptr %child.scope.slot
  %unattached = icmp eq ptr %child.scope, null
  br i1 %unattached, label %link, label %enqueue
link:
  %children.slot = getelementptr inbounds %princi.rt.scope, ptr %scope, i32 0, i32 1
  %children = load ptr, ptr %children.slot
  %scope.next.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 10
  store ptr %children, ptr %scope.next.slot
  store ptr %scope, ptr %child.scope.slot
  store ptr %task, ptr %children.slot
  br label %enqueue
enqueue:
  call void @princi_rt_task_enqueue(ptr %task)
  ret void
}

define internal ptr @princi_rt_task_await(ptr %task) {
entry:
  %state.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 0
  %state = load i32, ptr %state.slot
  %complete = icmp eq i32 %state, 4
  br i1 %complete, label %return, label %suspend
suspend:
  %current = load ptr, ptr @.princi.rt.async.current
  %no.current = icmp eq ptr %current, null
  %self.await = icmp eq ptr %current, %task
  %invalid = or i1 %no.current, %self.await
  br i1 %invalid, label %panic, label %start
panic:
  %message = getelementptr inbounds [37 x i8], ptr @.princi.rt.async.deadlock, i64 0, i64 0
  call void @princi_rt_task_panic(ptr %message)
  unreachable
start:
  %created = icmp eq i32 %state, 0
  br i1 %created, label %enqueue, label %wait
enqueue:
  call void @princi_rt_task_enqueue(ptr %task)
  br label %wait
wait:
  %waiting.slot = getelementptr inbounds %princi.rt.task, ptr %current, i32 0, i32 8
  store ptr %task, ptr %waiting.slot
  %current.state.slot = getelementptr inbounds %princi.rt.task, ptr %current, i32 0, i32 0
  store i32 3, ptr %current.state.slot
  %scheduler = load ptr, ptr @.princi.rt.async.scheduler
  call void @SwitchToFiber(ptr %scheduler)
  br label %return
return:
  %result.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 4
  %result = load ptr, ptr %result.slot
  ret ptr %result
}

define internal ptr @princi_rt_task_result(ptr %task) {
entry:
  %result.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 4
  %result = load ptr, ptr %result.slot
  ret ptr %result
}

define internal void @princi_rt_scope_enter(ptr %scope) {
entry:
  %current = load ptr, ptr @.princi.rt.async.current
  %task.scope.slot = getelementptr inbounds %princi.rt.task, ptr %current, i32 0, i32 9
  %parent = load ptr, ptr %task.scope.slot
  %parent.slot = getelementptr inbounds %princi.rt.scope, ptr %scope, i32 0, i32 0
  store ptr %parent, ptr %parent.slot
  %children.slot = getelementptr inbounds %princi.rt.scope, ptr %scope, i32 0, i32 1
  store ptr null, ptr %children.slot
  store ptr %scope, ptr %task.scope.slot
  ret void
}

define internal void @princi_rt_scope_exit(ptr %scope) {
entry:
  %children.slot = getelementptr inbounds %princi.rt.scope, ptr %scope, i32 0, i32 1
  %first = load ptr, ptr %children.slot
  br label %loop
loop:
  %child = phi ptr [ %first, %entry ], [ %next, %joined ]
  %empty = icmp eq ptr %child, null
  br i1 %empty, label %restore, label %join
join:
  %next.slot = getelementptr inbounds %princi.rt.task, ptr %child, i32 0, i32 10
  %next = load ptr, ptr %next.slot
  %ignored = call ptr @princi_rt_task_await(ptr %child)
  br label %joined
joined:
  br label %loop
restore:
  %parent.slot = getelementptr inbounds %princi.rt.scope, ptr %scope, i32 0, i32 0
  %parent = load ptr, ptr %parent.slot
  %current = load ptr, ptr @.princi.rt.async.current
  %task.scope.slot = getelementptr inbounds %princi.rt.task, ptr %current, i32 0, i32 9
  store ptr %parent, ptr %task.scope.slot
  ret void
}


define internal ptr @princi_rt_task_run_main(ptr %main.task) {
entry:
  call void @princi_rt_task_enqueue(ptr %main.task)
  br label %loop
loop:
  %main.state.slot = getelementptr inbounds %princi.rt.task, ptr %main.task, i32 0, i32 0
  %main.state = load i32, ptr %main.state.slot
  %main.done = icmp eq i32 %main.state, 4
  br i1 %main.done, label %done, label %schedule
schedule:
  %task = call ptr @princi_rt_task_dequeue()
  %empty = icmp eq ptr %task, null
  br i1 %empty, label %deadlock, label %run
deadlock:
  %message = getelementptr inbounds [37 x i8], ptr @.princi.rt.async.deadlock, i64 0, i64 0
  call void @princi_rt_task_panic(ptr %message)
  unreachable
run:
  store ptr %task, ptr @.princi.rt.async.current
  %state.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 0
  store i32 2, ptr %state.slot
  %fiber.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 1
  %fiber = load ptr, ptr %fiber.slot
  call void @SwitchToFiber(ptr %fiber)
  %state = load i32, ptr %state.slot
  %completed = icmp eq i32 %state, 4
  br i1 %completed, label %wake, label %continue
wake:
  call void @princi_rt_task_wake_waiters(ptr %task)
  br label %continue
continue:
  store ptr null, ptr @.princi.rt.async.current
  br label %loop
done:
  %result.slot = getelementptr inbounds %princi.rt.task, ptr %main.task, i32 0, i32 4
  %result = load ptr, ptr %result.slot
  ret ptr %result
}

define internal void @princi_rt_task_runtime_shutdown() {
entry:
  %first = load ptr, ptr @.princi.rt.async.tasks
  br label %loop
loop:
  %task = phi ptr [ %first, %entry ], [ %next, %continue ]
  %finished = icmp eq ptr %task, null
  br i1 %finished, label %convert, label %release
release:
  %next.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 6
  %next = load ptr, ptr %next.slot
  %fiber.slot = getelementptr inbounds %princi.rt.task, ptr %task, i32 0, i32 1
  %fiber = load ptr, ptr %fiber.slot
  %has.fiber = icmp ne ptr %fiber, null
  br i1 %has.fiber, label %delete, label %continue
delete:
  call void @DeleteFiber(ptr %fiber)
  br label %continue
continue:
  br label %loop
convert:
  %scheduler = load ptr, ptr @.princi.rt.async.scheduler
  %has.scheduler = icmp ne ptr %scheduler, null
  br i1 %has.scheduler, label %to.thread, label %reset
to.thread:
  %ignored = call i32 @ConvertFiberToThread()
  br label %reset
reset:
  store ptr null, ptr @.princi.rt.async.scheduler
  store ptr null, ptr @.princi.rt.async.current
  store ptr null, ptr @.princi.rt.async.tasks
  store ptr null, ptr @.princi.rt.async.ready.head
  store ptr null, ptr @.princi.rt.async.ready.tail
  ret void
}

"#;

pub(crate) fn create_function() -> &'static str {
    "princi_rt_task_create"
}
pub(crate) fn spawn_function() -> &'static str {
    "princi_rt_task_spawn"
}
pub(crate) fn await_function() -> &'static str {
    "princi_rt_task_await"
}
pub(crate) fn result_function() -> &'static str {
    "princi_rt_task_result"
}
pub(crate) fn scope_enter_function() -> &'static str {
    "princi_rt_scope_enter"
}
pub(crate) fn scope_exit_function() -> &'static str {
    "princi_rt_scope_exit"
}
pub(crate) fn run_main_function() -> &'static str {
    "princi_rt_task_run_main"
}
pub(crate) fn runtime_init_function() -> &'static str {
    "princi_rt_task_runtime_init"
}
pub(crate) fn runtime_shutdown_function() -> &'static str {
    "princi_rt_task_runtime_shutdown"
}

#[cfg(test)]
mod tests {
    use super::ASYNC_RUNTIME;

    #[test]
    fn async_runtime_keeps_the_private_fiber_scheduler_abi_complete() {
        for required in [
            "declare ptr @ConvertThreadToFiber(ptr)",
            "declare ptr @CreateFiber(i64, ptr, ptr)",
            "declare void @SwitchToFiber(ptr)",
            "declare void @DeleteFiber(ptr)",
            "declare i32 @ConvertFiberToThread()",
            "State values: created=0, ready=1, running=2, pending=3, completed=4.",
            "define internal ptr @princi_rt_task_create",
            "define internal ptr @princi_rt_task_await",
            "define internal void @princi_rt_task_wake_waiters",
            "define internal ptr @princi_rt_task_run_main",
            "define internal void @princi_rt_task_runtime_shutdown",
        ] {
            assert!(
                ASYNC_RUNTIME.contains(required),
                "missing runtime ABI item: {required}"
            );
        }
    }
}
