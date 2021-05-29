pub mod context;

use spin::Mutex;
use array_macro::array;
use core::ptr;

use context::Context;
use crate::memory::{KSTACK, addr::VirtualAddr, addr::Addr};
use crate::register::{tp, sstatus};

use crate::consts::{NPROC, NCPU,PGSIZE};

static next_pid: Mutex<isize> = spin::Mutex::new(0);
static mut proc_list: [Proc; NPROC] = array![_ => Proc::new(); NPROC];
static mut cpu_list: [Cpu; NCPU] = array![_ => Cpu::new(); NCPU];


enum ProcState{ UNUSED, USED, SLEEPING, RUNNABLE, RUNNING, ZOMBIE }

pub struct Proc {
    lock: spin::Mutex<bool>,
    kstack: VirtualAddr,
    state: ProcState,
    pid: isize,
    context: Context,
}

impl Proc {
    pub const fn new() -> Proc {
        Proc {
            state: ProcState::UNUSED,
            lock: spin::Mutex::new(false),
            kstack: VirtualAddr::from(0),
            pid: -1,
            context: Context::new(),
        }
    }

    pub fn set_kstack(&mut self, addr: VirtualAddr) {
        self.kstack = addr;
    }
}

pub struct Cpu {
    proc_index: isize,
    noff: usize,
    intena: bool,
    scheduler: Context,
}

impl Cpu {
    pub const fn new() -> Cpu {
        Cpu {
            proc_index: -1,
            scheduler: Context::new(),
            noff: 0,
            intena: false,
        }
    }
}

/// push_off/pop_off are like intr_off()/intr_on() except that they are matched:
/// it takes two pop_off()s to undo two push_off()s.  Also, if interrupts
/// are initially off, then push_off, pop_off leaves them off.
unsafe fn push_off() {
    let old = sstatus::intr_get();
    sstatus::intr_off();
    let cpu_index = my_cpu_index();
    if cpu_list[cpu_index].noff == 0 {
        cpu_list[cpu_index].intena = old;
    }
    cpu_list[cpu_index].noff +=1 ;
}

unsafe fn pop_off() {
    let cpu_index = my_cpu_index();
    if sstatus::intr_get() {
        panic!("pop_off(): interruptable");
    }
    if cpu_list[cpu_index].noff < 1 {
        panic!("pop_off(): count not match");
    }
    cpu_list[cpu_index].noff -= 1;
    if cpu_list[cpu_index].noff == 0 && cpu_list[cpu_index].intena {
        sstatus::intr_on();
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

/// cpuid() has already been implemented in `crate::register`
unsafe fn cpu_id() -> usize{
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
    let index = cpu_list[my_cpu_index()].proc_index;
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
        let raii = proc_iter.lock.lock();
        match proc_iter.state{
            ProcState::UNUSED =>{
                proc_iter.pid = alloc_pid();
                proc_iter.state = ProcState::USED;
                // TODO : add trapframe
                // TODO : page table
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