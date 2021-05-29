pub mod context;
pub mod cpu;

use spin::Mutex;
use array_macro::array;
use core::ptr;

use context::Context;
use alloc::boxed::Box;
use cpu::{Cpu,push_off,pop_off};
use crate::memory::{KSTACK, addr::VirtualAddr, addr::Addr, uvmfree, uvmcreate, pagetable::PageTable};
use crate::register::tp;

use crate::consts::{NPROC, NCPU,PGSIZE};

static next_pid: Mutex<isize> = spin::Mutex::new(0);
static mut proc_list: [Proc; NPROC] = array![_ => Proc::new(); NPROC];
static mut cpu_list: [Cpu; NCPU] = array![_ => Cpu::new(); NCPU];


enum ProcState{ UNUSED, USED, SLEEPING, RUNNABLE, RUNNING, ZOMBIE }

pub struct Proc {
    lock: spin::Mutex<bool>,
    kstack: VirtualAddr,
    state: ProcState,
    sz: usize,
    pid: isize,
    context: Context,
    pagetable: * mut PageTable,
}

impl Proc {
    pub const fn new() -> Proc {
        Proc {
            state: ProcState::UNUSED,
            lock: spin::Mutex::new(false),
            kstack: VirtualAddr::from(0),
            pid: -1,
            sz: 0,
            context: Context::new(),
            pagetable: ptr::null_mut(),
        }
    }

    pub fn set_kstack(&mut self, addr: VirtualAddr) {
        self.kstack = addr;
    }
    pub unsafe fn generate_pagetable(&mut self){
        let pagetable: Box<PageTable>= uvmcreate();
        // TODO trapoline 
        //TODO trapframe
        self.pagetable = Box::into_raw(pagetable);
    }
    pub unsafe fn freepagetable(&mut self){
        //TODO trampoline uvmunmap();
        //TODO trapframe uvmunmap();
        uvmfree(&mut *self.pagetable, self.sz);
    }
    pub unsafe fn freeproc(&mut self){
        self.state= ProcState::UNUSED;
        self.lock= spin::Mutex::new(false);
        self.kstack.set(0);
        self.pid= -1;
        self.sz= 0;
        self.context.clear();
        self.pagetable= ptr::null_mut();
    }
}


pub unsafe fn proc_init() {
    let mut index: usize = 0;
    for proc_iter in proc_list.iter_mut() {
        proc_iter.set_kstack(VirtualAddr::from(KSTACK(index)));
        index += 1;
    }
    println!("process init finished!");
}

pub unsafe fn cpu_id() -> usize{
    tp::read()
}

/// Return this CPU's index.
/// Interrupts must be disabled.
unsafe fn my_cpu_index() -> usize {
    cpu_id()
}

/// Return the current process's index
/// -1 if none
unsafe fn my_proc_index() -> isize {
    push_off();
    let index = cpu_list[my_cpu_index()].get_proc_index();
    pop_off();
    index
}

fn alloc_pid() -> isize {
    let mut nextpid = *(next_pid.lock());
    let pid = nextpid;
    nextpid += 1;
    pid
}

unsafe fn alloc_proc_index() -> isize {
    let mut count : isize = 0;
    for proc_iter in proc_list.iter_mut() {
        //consider to add a lock here
        //let raii = proc_iter.lock.lock();
        match proc_iter.state{
            ProcState::UNUSED =>{
                proc_iter.pid = alloc_pid();
                proc_iter.state = ProcState::USED;
                // TODO : add trapframe
                proc_iter.generate_pagetable();
                proc_iter.context.set_ra(forkret as *const () as usize);
                proc_iter.context.set_sp(*proc_iter.kstack.add(PGSIZE).get_self());
                return count;
            },
            _ => ()
        }
        count += 1 ;
    }
    return -1;
}

// A fork child's very first scheduling by scheduler()
// will swtch to forkret.
fn forkret() {

}