use core::ptr;

use crate::config::{MAX_STACK_SIZE, MAX_TASKS, SYSTICK_INTERRUPT_INTERVAL};
use crate::errors::KernelError;
use crate::interrupt_handlers::svc_call;
use crate::kernel::helper::get_msb;
use crate::process::*;
use cortex_m::interrupt::free as execute_critical;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::register::control::Npriv;

use crate::kernel::types::TaskId;

#[repr(C)]
pub struct Scheduler {
    pub curr_pid: usize,
    pub is_running: bool,
    pub threads: [Option<TaskControlBlock>; MAX_TASKS],
    pub blocked_tasks: u32,
    pub active_tasks: u32,
    pub is_preemptive: bool,
    pub started: bool,
}

/// A single thread's state
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TaskControlBlock {
    // fields used in assembly, do not reorder them
    pub sp: usize, // current stack pointer of this thread
}

#[no_mangle]
static mut TASK_STACKS: [[u32; MAX_STACK_SIZE]; MAX_TASKS] = [[0; MAX_STACK_SIZE]; MAX_TASKS];

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            curr_pid: 0,
            is_running: false,
            threads: [None; MAX_TASKS],
            active_tasks: 1,
            blocked_tasks: 0,
            is_preemptive: false,
            started: false,
        }
    }

    /// Initialize the switcher system
    pub fn init(&mut self, is_preemptive: bool) {
        self.is_preemptive = is_preemptive;
        /*
            This is the default task, that just puts the board for a power-save mode
            until any event (interrupt/exception) occurs.
        */
        self.create_task(
            0,
            |_| loop {
                cortex_m::asm::wfe();
            },
            &0,
        )
        .unwrap();
    }

    // The below section just sets up the timer and starts it.
    pub fn start_kernel(&mut self) -> Result<(), KernelError> {
        self.is_running = true;
        Ok(())
    }

    pub fn create_task<T: Sized>(
        &mut self,
        priority: usize,
        handler_fn: fn(&T) -> !,
        param: &T,
    ) -> Result<(), KernelError> {
        let mut stack = unsafe { &mut TASK_STACKS[priority] };
        match self.create_tcb(stack, handler_fn, param) {
            Ok(tcb) => {
                self.insert_tcb(priority, tcb)?;
                return Ok(());
            }
            Err(e) => return Err(e),
        }
    }

    fn create_tcb<T: Sized>(
        &self,
        stack: &mut [u32],
        handler: fn(&T) -> !,
        param: &T,
    ) -> Result<TaskControlBlock, KernelError> {
        if stack.len() < 32 {
            return Err(KernelError::StackTooSmall);
        }

        let idx = stack.len() - 1;
        let args: u32 = unsafe { core::intrinsics::transmute(param) };
        let pc: usize = handler as usize;

        stack[idx] = 1 << 24; // xPSR
        stack[idx - 1] = pc as u32; // PC
        stack[idx - 7] = args; // args

        let sp: usize = unsafe { core::intrinsics::transmute(&stack[stack.len() - 16]) };
        let tcb = TaskControlBlock { sp: sp as usize };

        Ok(tcb)
    }

    fn insert_tcb(&mut self, idx: usize, tcb: TaskControlBlock) -> Result<(), KernelError> {
        if idx >= MAX_TASKS {
            return Err(KernelError::DoesNotExist);
        }
        self.threads[idx] = Some(tcb);
        return Ok(());
    }

    pub fn block_tasks(&mut self, tasks_mask: u32) {
        self.blocked_tasks |= tasks_mask;
    }

    pub fn unblock_tasks(&mut self, tasks_mask: u32) {
        self.blocked_tasks &= !tasks_mask;
    }

    pub fn get_HT(&self) -> usize {
        let mask = self.active_tasks & !self.blocked_tasks;
        return get_msb(&mask);
    }

    pub fn release(&mut self, tasks_mask: &u32) {
        self.active_tasks |= *tasks_mask;
    }
}