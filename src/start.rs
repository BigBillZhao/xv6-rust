use crate::register::{startup, mepc, satp, mstatus, mie, mhartid, clint, mscratch, mtvec};
use crate::rmain::rust_main;

use crate::consts::{NCPU, CLINT_MTIMECMP};

/// for each cpu, only 6 of 32 usize are used, others are reserved.
static mut MSCRATCH0: [usize; NCPU * 32] = [0; NCPU * 32];

#[no_mangle]
pub unsafe fn start() -> ! {
    //set M Previous Privilege mode to Supervisor, for mret.
    startup::mstatus();

    //set M Execption Program Counter to main , for mret.
    // requires gcc -mcmodel=medany
    mepc::write(rust_main as usize);

    //disable paging
    satp::write(0);

    //delegate all interrupts and exceptions to supervisor mode
    startup::to_supervisor();

    //ask for clock interrupts
    timerinit();

    //give cpu id
    startup::allo_cpu();

    //switch to supervisor mode and jump to main()
    llvm_asm!("mret"::::"volatile");

    loop {}
}

unsafe fn timerinit(){
    // each CPU has a separate source of timer interrupts
    let id = mhartid::read();

    // ask the CLINT for a timer interrupt.
    let interval: u64 = 1000000; // cycles; about 1/10th second in qemu.
    clint::add_mtimecmp(id, interval);

    // prepare information in scratch[] for timervec.
    // scratch[0..3] : space for timervec to save registers.
    // scratch[4] : address of CLINT MTIMECMP register.
    // scratch[5] : desired interval (in cycles) between timer interrupts.
    let offset = 32 * id;
    MSCRATCH0[offset + 4] = 8 * id + CLINT_MTIMECMP;
    MSCRATCH0[offset + 5] = interval as usize;
    mscratch::write((MSCRATCH0.as_ptr() as usize) + offset * core::mem::size_of::<usize>());

    // set the machine-mode trap handler.
    extern "C" {
        fn timervec();
    }
    mtvec::write(timervec as usize);

    // enable machine-mode interrupts.
    mstatus::set_mie();

    // enable machine-mode timer interrupts.
    mie::set_mtie();
}