use core::ptr;
use crate::proc::context::Context;
use crate::register::sstatus;
use crate::proc::proc::Proc;
use crate::proc::proc;
use crate::consts::NCPU;
pub static mut cpu_list: &[Cpu] = &[Cpu::Cpu(); NCPU];

pub struct Cpu<'a> {
    pub proc: *mut Proc<'a>,
    pub noff: usize,
    pub intena: bool,
    pub scheduler: Context,
}

impl Cpu<'_> {
    const fn Cpu() -> Self {
        Self {
            proc: ptr::null_mut(),
            scheduler: Context::Context(),
            noff: 0,
            intena: false,
        }
    }
}
/// push_off/pop_off are like intr_off()/intr_on() except that they are matched:
/// it takes two pop_off()s to undo two push_off()s.  Also, if interrupts
/// are initially off, then push_off, pop_off leaves them off.
pub fn push_off() {
    let old = sstatus::intr_get();
    sstatus::intr_off();
    if (*proc::mycpu()).noff==0 {
        (*proc::mycpu()).intena = old;
    }
    (*proc::mycpu()).noff +=1 ;
}

pub fn pop_off() {
    let c = proc::mycpu();
    if sstatus::intr_get() {
        panic!("pop_off(): interruptable");
    }
    if (*c).noff < 1 {
        panic!("pop_off(): count not match");
    }
    (*c).noff -= 1;
    if (*c).noff == 0 && (*c).intena {
        sstatus::intr_on();
    }
}
