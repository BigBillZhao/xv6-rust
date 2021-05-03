use crate::register::tp;

pub unsafe fn rust_main() -> ! {

    let cpuid = tp::read();
    if cpuid == 0{
        crate::console::console_init();
        println!();
        println!("hello world");
        panic!();
    } else {
        
    }
    loop {}
}