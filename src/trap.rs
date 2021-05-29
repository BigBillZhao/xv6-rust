use crate::register::stvec;

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

// !!! not inplemented corresponding initialization in `start.c`
fn clockintr () {

}

// check if it's an external interrupt or software interrupt,
// and handle it.
// returns 2 if timer interrupt,
// 1 if other device,
// 0 if not recognized.
fn devintr() -> usize {
    0
}
