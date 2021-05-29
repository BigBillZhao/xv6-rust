pub unsafe fn rust_main() -> ! {

    let cpuid = crate::register::tp::read();
    if cpuid == 0{
        crate::console::console_init();
        println!();
        println!("hello world");
        crate::memory::kalloc::kinit();
        crate::memory::kvminit();
        crate::memory::kvminithart();
        crate::proc::proc_init();
        panic!();
    } else {
        
    }
    loop {}
}