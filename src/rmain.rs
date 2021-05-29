pub unsafe fn rust_main() -> ! {

    let cpuid = crate::proc::cpu_id();
    if cpuid == 0{
        crate::console::console_init();
        println!();
        println!("hello world");
        crate::memory::kalloc::kinit();
        crate::memory::kvminit();
        crate::memory::kvminithart();
        crate::proc::proc_init();
        crate::plic::plic_init();
        crate::plic::plic_init_hart();
        crate::trap::trap_init_hart();
        panic!();
    } else {
        
    }
    loop {}
}