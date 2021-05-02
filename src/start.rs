//use crate::consts::param;
use crate::register::startup;
use crate::register;
use crate::rmain::rust_main;

#[no_mangle]
pub unsafe fn start() -> ! {
    //set M Previous Privilege mode to Supervisor, for mret.
    startup::mstatus();

    //set M Execption Program Counter to main , for mret.
    // requires gcc -mcmodel=medany
    register::mepc::write(rust_main as usize);

    //disable paging
    register::satp::write(0);

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

fn timerinit(){

}