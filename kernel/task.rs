extern crate alloc;
use alloc::vec::Vec;
use spin::Mutex;
use crate::memory::{self, make_user_accessible};
use x86_64::VirtAddr;
#[derive(Debug, Clone, Copy)]
#[repr(C)]

pub struct TaskContext {
    // the registers that have to stay while an functioncall
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
    rip: u64, // instruction pointers tell where the task will continue running
}

pub struct Task  {
    pub id: usize,
    pub stack_pointer: u64,
    // Prevents the stack allocated by the allocator from being freed
    pub _stack_mem: Vec<u8>,
}

pub struct Scheduler {
    // 2 tasks for: Kernel/Main = 0 and Shell = 1
    pub tasks: [Option<Task>; 2],
    pub current_task: usize,
}

pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler {
    tasks: [None, None],
    current_task: 0,
});

impl Task {
    
    
    pub fn new(id: usize, entry_point: fn() -> !) -> Self {
        let stack_size = 4096 * 4; // 16kb stack for shell
        let mut stack = alloc::vec![0u8; stack_size];

        let stack_start = stack.as_ptr() as u64;
        let mut rsp_top = stack_start + stack_size as u64; // stack grows downwards

        let mut rsp = rsp_top;

        // prepares stack for context switch
        unsafe {
            let mut rsp_ptr = rsp as *mut u64;

            // The interrupt stack frame that "iretq" expects at the end

            // Stack segment
            rsp_ptr = rsp_ptr.offset(-1);
            *rsp_ptr = 0;

            // RSP points onto the top of the stack before the interrupt
            rsp_ptr = rsp_ptr.offset(-1);
            *rsp_ptr = rsp_top;

            //  0x200 activates the interrupt flag so when this task is activated interrupts are automaticly on
            rsp_ptr = rsp_ptr.offset(-1);
            *rsp_ptr = 0x202;

            // 0x08 is the standart kernel code selector from the gdt
            rsp_ptr = rsp_ptr.offset(-1);
            *rsp_ptr = 0x08;

            // RIP the entry point of the function
            rsp_ptr = rsp_ptr.offset(-1);
            *rsp_ptr = entry_point as u64;

            for _ in 0..15 {
                rsp_ptr = rsp_ptr.offset(-1);
                *rsp_ptr = 0;
            }

            rsp = rsp_ptr as u64;
        }

        Task {
            id,
            stack_pointer: rsp,
            _stack_mem: stack,
        }
    }

    
    // we overgive the mapper as refference
    pub fn new_user(
        id: usize,
        entry_point: fn() -> !,
        mapper: &mut x86_64::structures::paging::OffsetPageTable
    ) -> Self {
        let stack_size = 4096 * 4;
        let mut stack = alloc::vec![0u8; stack_size];

        let stack_start = stack.as_ptr() as u64;
        let mut rsp_top = stack_start + stack_size as u64;

        // make the stack accesible for ring 3
        make_user_accessible(mapper, VirtAddr::new(stack_start), stack_size as u64);

        // making the programm code accsesible for ring 3
        make_user_accessible(mapper, VirtAddr::new(entry_point as u64), 4096);

        let mut rsp = rsp_top;

        unsafe {
            let mut rsp_ptr = rsp as *mut u64;

            // Stack Segment: User Data Selector + RPL 3
            rsp_ptr = rsp_ptr.offset(-1);
            *rsp_ptr = crate::gdt::GDT.1.user_data_selector.0 as u64 | 3;

            // RSP before the interrupt
            rsp_ptr = rsp_ptr.offset(-1);
            *rsp_ptr = rsp_top;

            // RFLAGS: 0x200 activates the Interrupt Flag, 0x3000 sets IOPL to 3
            rsp_ptr = rsp_ptr.offset(-1);
            *rsp_ptr = 0x3202;

            // Code Segment: User Code Selector + RPL 3
            rsp_ptr = rsp_ptr.offset(-1);
            *rsp_ptr = crate::gdt::GDT.1.user_code_selector.0 as u64 | 3;

            // RIP: Entry Point of the functions[cite: 7]
            rsp_ptr = rsp_ptr.offset(-1);
            *rsp_ptr = entry_point as u64;

            // making space for the 15 registers that get poped by the timer interrupt handler
            for _ in 0..15 {
                rsp_ptr = rsp_ptr.offset(-1);
                *rsp_ptr = 0;
            }

            rsp = rsp_ptr as u64;
        }


        Task {
            id,
            stack_pointer: rsp,
            _stack_mem: stack,
        }
    }


}

impl Scheduler {
    pub fn pick_next_task(&mut self, current_rsp: u64) -> u64 {
        // if the task isnt fully initalized stay in current context
        if self.tasks[0].is_none() || self.tasks[1].is_none() {
            return current_rsp;
        }

        let old_task = self.current_task;
        let next_task = (old_task + 1) % 2;

        // saves the current stack pointer in task object
        if let Some(ref mut task) = self.tasks[old_task] {
            task.stack_pointer = current_rsp;
        }

        // swap into the intern index
        self.current_task = next_task;

        // return the new stack pointer
        self.tasks[next_task].as_ref().unwrap().stack_pointer
    }
}