
extern "C" {
    fn trampoline();
}

/// Allocate a page for each process's kernel stack.
/// Map it high in memory, followed by an invalid
/// guard page.
pub fn proc_mapstacks(kpgtbl: usize){

}

pub fn procinit() {

}

// cpuid() has already been implemented in `crate::register`

// Return this CPU's cpu struct.
// Interrupts must be disabled.
pub fn mycpu() -> Struct cpu {

}

// Return the current struct proc *, or zero if none.
pub fn myproc() -> Struct proc {

}

fn allocpid() -> usize {

}

// Look in the process table for an UNUSED proc.
// If found, initialize state required to run in the kernel,
// and return with p->lock held.
// If there are no free procs, or a memory allocation fails, return 0.
fn allocproc() -> Struct proc {

}

// free a proc structure and the data hanging from it,
// including user pages.
// p->lock must be held.
fn freeproc(p: & Struct proc) {

}

// Create a user page table for a given process,
// with no user memory, but with trampoline pages.
fn proc_pagetable(p: & Struct proc) -> usize {

}

// Free a process's page table, and free the
// physical memory it refers to.
fn proc_freepagetable(pagetable: usize, size: usize) {

}

// uchar initcode[] {...}

pub fn userinit() {

}

// Grow or shrink user memory by n bytes.
// Return 0 on success, -1 on failure.
fn growproc(n: usize) -> bool {

}

fn fork() -> usize {

}

fn reparent(p: & Struct proc) {

}

fn exit(status: isize) {

}

// Wait for a child process to exit and return its pid.
// Return -1 if this process has no children.
fn wait(addr: usize) -> isize {

}

// Per-CPU process scheduler.
// Each CPU calls scheduler() after setting itself up.
// Scheduler never returns.  It loops, doing:
//  - choose a process to run.
//  - swtch to start running that process.
//  - eventually that process transfers control
//    via swtch back to the scheduler.
fn scheduler() {

}

fn yield() {

}

// A fork child's very first scheduling by scheduler()
// will swtch to forkret.
fn forkret() {

}

fn sleep(chan: usize) {

}

fn wakeup(chan: usize) {

}

fn kill(pid: usize) -> isize {

}

fn either_copyout() { }
fn either_copyin() { }

pub fn procdump() {
    
}