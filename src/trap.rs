use crate::register::stvec;
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

#[no_mangle]
pub extern "C" fn kerneltrap() {

}

fn clockintr () {
    let mut _ticks = *(TICKS.lock());
    println!("timer interrupt tick = {}", _ticks);
    _ticks += 1;
    drop(_ticks);
}

// check if it's an external interrupt or software interrupt,
// and handle it.
// returns 2 if timer interrupt,
// 1 if other device,
// 0 if not recognized.
fn devintr() -> usize {
    2
}
