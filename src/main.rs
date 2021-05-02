#![no_std]
#![no_main]

#[allow(unused_imports)]
use xv6_rust_project;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
