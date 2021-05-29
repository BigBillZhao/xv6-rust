use super::context::Context;
use super::{cpu_list,my_cpu_index};
use crate::register::sstatus;

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
    pub fn get_proc_index(&self)->isize{
        self.proc_index
    }
}

/// push_off/pop_off are like intr_off()/intr_on() except that they are matched:
/// it takes two pop_off()s to undo two push_off()s.  Also, if interrupts
/// are initially off, then push_off, pop_off leaves them off.
pub unsafe fn push_off() {
    let old = sstatus::intr_get();
    sstatus::intr_off();
    let cpu_index = my_cpu_index();
    if cpu_list[cpu_index].noff == 0 {
        cpu_list[cpu_index].intena = old;
    }
    cpu_list[cpu_index].noff +=1 ;
}

pub unsafe fn pop_off() {
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