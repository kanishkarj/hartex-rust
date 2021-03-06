//! Macro Definitions

/// The tasks must be looping infinitely and call `task_exit` whenever a particular task is done.
/// This makes it complicated to create tasks and also might introduce undefined behavior if task_exit is not called.
/// The `spawn` macro makes it easier to define tasks. It also defines a static variable of type TaskId,
/// which corresponds to the task created.
///
/// ## Examples
///
/// ```rust
/// shared = 10;
/// spawn!(task2, 2, stack1, shared, params, {
///     hprintln!("{}", params);
/// });
/// spawn!(task3, 3, stack2, {
///     hprintln!("Hello!");
/// });
/// ```
#[macro_export]
macro_rules! spawn {
    ($priority: expr, $stack_size: ident, $handler_fn: tt) => {
        let mut stack = [0; $stack_size];
        create_task(
            $priority,
            unsafe { &mut stack },
            |cxt: ContextType| loop {
            $handler_fn(cxt);
                task_exit();
            },
        );
    };
    ($priority: expr, $deadline: expr, $stack_size: ident, $handler_fn: tt) => {
        let mut stack = [0; $stack_size]
        create_task(
            $priority,
            $deadline,
            unsafe { &mut stack },
            |cxt: ContextType| loop {
                $handler_fn(cxt);
                task_exit();
            },
        );
    };
}

/// `priv_execute!` executes the code block only if the current context is in privileged mode.
/// ## Example
/// ```rust
/// priv_execute!({
///     hprintln!("Privileged!");
/// });
/// ```
#[macro_export]
macro_rules! priv_execute {
    ($handler: block) => {
        match is_privileged() {
            false => Err(KernelError::AccessDenied),
            true => $handler,
        }
    };
}
