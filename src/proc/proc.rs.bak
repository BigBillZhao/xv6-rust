use spin::Mutex;
use core::ptr;
use crate::proc::trapframe::TrapFrame;
use crate::consts::param::{NPROC,NCPU};
use crate::consts::vm::PGSIZE;
use crate::consts::{PTE_V, PTE_R, PTE_W, PTE_X, PTE_U};
use crate::consts::memlayout::TRAMPOLINE;
use crate::memory::kalloc::kalloc;
use crate::memory::vm;
use crate::register::tp;
use crate::proc::context::Context;
use crate::proc::cpu::Cpu;
use crate::proc::cpu;
enum ProcState{ UNUSED, USED, SLEEPING, RUNNABLE, RUNNING, ZOMBIE }

pub struct Proc<'a>{
    state: ProcState,
    chan: *mut Proc<'a>,
    killed: bool,
    xstate: isize,//exist status to parent 
    pid: isize,
    context: Context,
    parent: *mut Proc<'a>,

    pub kstack: usize,
    sz: usize,
    pagetable: usize,
    trapframe: TrapFrame,
    //ofile: this is open files
    //cwd: this is current directory
    name: &'a str,

}
impl Proc<'static>{
    fn Proc()-> Self{
        Self{
            state: ProcState::UNUSED,
            chan: ptr::null_mut(),
            killed: false,
            xstate: 0,//exist status to parent 
            pid: 0,

            parent: ptr::null_mut(),

            kstack: 0,
            sz: 0,
            pagetable: 0,
            trapframe: TrapFrame::TrapFrame(),
            context: Context::Context(),
            //ofile: this is open files
            //cwd: this is current directory
            name: &"nothing",
        }
    }
    // Grow or shrink user memory by n bytes.
    // Return 0 on success, -1 on failure.
    fn growproc(&self,n: usize) -> bool {
        let mut size = self.sz;
        if n>0{
            size = vm::uvmalloc(self.pagetable,size,size+n);
            if size == 0 {
                return false;
            }
        }else if n<0{
            size = vm::uvmdealloc(self.pagetable,size,size+n);
        }
        self.sz =size;
        return true;
    }

    fn fork(&mut self) -> isize {
        let np = match allocproc(){
            Some(np) => np,
            None => panic!()
        };

        if vm::uvmcopy(self.pagetable,np.pagetable,self.sz){
            freeproc(np);
            return -1;
        }
        np.sz = self.sz;
        np.trapframe = self.trapframe;
        np.trapframe.a0=0;

        //open file descriptors
        //haven't implement yet

        np.name = self.name;

        np.parent = self;
        np.state = ProcState::RUNNABLE;

        let pid = np.pid;
        return pid;
    }
    //pass this process abandon child to init
    fn reparent(&mut self) {
        for proc in proc_list.iter(){
            let mut process = proc.lock();
            if process.parent== self{
                process.parent = initproc as *mut Proc<'static>;
                wakeup(initproc as *mut Proc<'static>);
            }
        }
    }

    fn exit(&self,status: isize) {
        if self == initproc{
            panic!("init exiting");
        }
        // close all open files 
        //TODO
        //some other file operation
        self.reparent();
        wakeup(self.parent);
        self.xstate=status;
        self.state=ProcState::ZOMBIE;
        sched();
        panic!("zombie exit");
    }
    fn sleep(&self,chan: *mut Proc<'static>) {
        //TODO: rethink about this
        //is there need another lock?
        self.chan=chan;
        self.state= ProcState::SLEEPING;
        sched();
        self.chan=ptr::null_mut();
    }
    

    
    
    // Wait for a child process to exit and return its pid.
    // Return -1 if this process has no children.
    // rethink about this , do this later
    /*
    fn wait(&self,addr: usize) -> isize {
        while true{
            let mut havekids = false;
            for proc :proc_list{
                let mut process = proc.lock();
                if process.parent==self {
                    havekids=true;
                    if  addr!=0 && copyout(self.pagetable,addr,process.xstate,sizeof(process.xstate)) {
                        //TODO : release a lock here
                        return -1;
                    }
                    freeproc(process);
                    //release a lock here
                    return pid;
                }
            }   
        }
        if !havekids || self.killed{
            //release a lock here
            return -1;
        }
        //figure out the input
        //the input here is a lock
        self.sleep();
    }
    */
}

extern "C" {
    fn trampoline();
}



//these are the lists that are static mut which are unsafe
static mut initproc: &Proc= &Proc::Proc();
static next_pid: Mutex<isize> = spin::Mutex::new(0);
static mut proc_list: &[Mutex<Proc>] = &[spin::Mutex::new(Proc::Proc()) ; NPROC];



/// Allocate a page for each process's kernel stack.
/// Map it high in memory, followed by an invalid
/// guard page.
pub fn proc_mapstacks(kpgtbl: usize){
    for i in 0..NPROC{
        let pa: usize= kalloc();
        if pa == 0{
            panic!("proc mapstacks kalloc");
        }
        let va: usize=KSTACK(i);
        vm::kvmmap(kpgtbl, va, pa, PGSIZE,PTE_R | PTE_W);
    }
}

pub fn procinit() {
    let mut count: usize=0;
    for p in proc_list.iter(){
        let mut process = p.lock();
        process.kstack=KSTACK(count);
        count=count+1;
    }
}

// cpuid() has already been implemented in `crate::register`
fn cpuid()->usize{
    tp::read()
}
// Return this CPU's cpu struct.
// Interrupts must be disabled.

//should we return a pointer here ?
pub fn mycpu() -> *mut Cpu<'static> {
    let id = cpuid();
    return &cpu::cpu_list[id] as *mut Cpu<'static>;
}

// Return the current struct proc *, or zero if none.
pub fn myproc() -> *mut Proc<'static> {
    cpu::push_off();
    let c = mycpu();
    let p = (*c).proc;
    cpu::pop_off();
    return p;
}

//see https://docs.rs/spin/0.4.5/spin/struct.Mutex.html
fn allocpid() -> isize {
    let mut nextpid = *(next_pid.lock());
    let pid= nextpid;
    nextpid +=1;
    return pid;
}

// Look in the process table for an UNUSED proc.
// If found, initialize state required to run in the kernel,
// and return with p->lock held.
// If there are no free procs, or a memory allocation fails, return 0.
fn allocproc() -> Option<&'static mut Proc<'static>> {
    for process in proc_list.iter(){
        let mut p = *(process.lock());
        if p.state == ProcState::UNUSED{
            p.pid= allocpid();
            p.state = ProcState::USED;
            // TODO: this place need to reconsider
            // need to transfer the pointer to the struct
            unsafe{
                p.trapframe = *(kalloc() as *mut TrapFrame);
            } 
            //user page table
            p.pagetable= proc_pagetable(&p);
            if p.pagetable == 0 {
                freeproc(&p);
                return None;
            }
            //memset(&p->context, 0, sizeof(p->context));
            // write it with 0, I think we don't need this
            p.context.set_ra(forkret as *const () as usize);
            p.context.set_sp(p.kstack+PGSIZE);
            return Some(&mut p);
        }
    }
    return None;
}

// free a proc structure and the data hanging from it,
// including user pages.
// p->lock must be held.
fn freeproc(p: & Proc) {
    // do we really need to free the process?
}

// Create a user page table for a given process,
// with no user memory, but with trampoline pages.
fn proc_pagetable(p: & Proc) -> usize {
    return 0;
}

// Free a process's page table, and free the
// physical memory it refers to.
fn proc_freepagetable(pagetable: usize, size: usize) {

}

// uchar initcode[] {...}

pub fn userinit() {

}

fn kill(pid: isize) -> bool {
    for proc in proc_list.iter(){
        let mut process = proc.lock();
        if process.pid==pid{
            process.killed=true;
            if process.state==ProcState::SLEEPING{
                process.state= ProcState::RUNNABLE;
            }
            return true;
        }
    }
    return false;   
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
fn sched(){

}
fn yield_proc() {

}

// A fork child's very first scheduling by scheduler()
// will swtch to forkret.
fn forkret() {

}

fn wakeup(chan: *mut Proc) {
    for proc in proc_list.iter(){
        let mut process = proc.lock();
        if process != myproc(){
            if process.state== ProcState::SLEEPING && process.chan== chan{
                process.state=ProcState::RUNNABLE;
            }
        }
    }   
}

fn either_copyout() { }
fn either_copyin() { }

pub fn procdump() {
    
}

fn KSTACK(p: usize)->usize{
    TRAMPOLINE - ((p+1)*2*PGSIZE)
}