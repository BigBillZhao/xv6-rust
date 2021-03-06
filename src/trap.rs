use crate::register::stvec;
use crate::register::{sstatus, scause, sepc, sip};
use crate::proc::{ProcState, my_proc_index, yield_proc, proc_list, cpu_id};
use crate::plic::plic_claim;
use spin::Mutex;

static TICKS: Mutex<usize> = Mutex::new(0);

pub unsafe fn trap_init_hart() {
    // in kernelvec.S, calls kerneltrap().
    // refered in trapinithalt and usertrap
    extern "C" {
        fn kernelvec();
    } 
    stvec::write(kernelvec as usize);
}

// handle an interrupt, exception, or system call from user space.
// called from trampoline.S
#[no_mangle]
pub extern "C" fn usertrap() {

}

fn usertrapret() {

}

const SPP: usize = 1 << 8;  // previous mode, is from supervisor?

#[no_mangle]
pub unsafe extern "C" fn kerneltrap() {
    let sepc =sepc::read();
    let sstatus = sstatus::read();
    let scause = scause::read();

    if sstatus & SPP == 0{
        panic!("not from supervisor mode");
    }
    if sstatus::intr_get(){
        panic!("interrupts enabled");
    }
    let dev=devintr();
    if dev==0 {
        panic!("kerneltrap");
    }
    if dev == 2 && my_proc_index() != -1 {
        match proc_list[my_proc_index() as usize].get_state(){
            RUNNING =>{
                yield_proc();
            },
            _ => ()
        }
    }
    sepc::write(sepc);
    sstatus::write(sstatus);
}

fn clockintr() {
    let mut _ticks = *(TICKS.lock());
    _ticks += 1;
    drop(_ticks);
}

// check if it's an external interrupt or software interrupt,
// and handle it.
// returns 2 if timer interrupt,
// 1 if other device,
// 0 if not recognized.
unsafe fn devintr() -> usize {
    let scause = scause::read();

    if (scause & 0x8000000000000000 != 0) && (scause & 0xff) == 9 {
        let irq = plic_claim();
        println!("interrupt irq is {}", irq);
        return 1;
    } else if scause == 0x8000000000000001 {
        if cpu_id() == 0 {
            clockintr();
        }
        let old = sip::read();
        sip::write(old & !2);
        return 2;
    }
    0
}
