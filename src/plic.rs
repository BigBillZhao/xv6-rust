const SENABLE: usize = 0x2080;
const SENABLE_HART: usize = 0x100;
const SPRIORITY: usize = 0x201000;
const SPRIORITY_HART: usize = 0x2000;
const SCLAIM: usize = 0x201004;
const SCLAIM_HART: usize = 0x2000;

use core::ptr;
use crate::consts::{PLIC, UART0_IRQ, VIRTIO0_IRQ};
use crate::proc::cpu_id;

pub unsafe fn plic_init() {
    // set desired IRQ priorities non-zero (otherwise disabled)
    write(UART0_IRQ*4, 1);
    write(VIRTIO0_IRQ*4, 1);
}

pub unsafe fn plic_init_hart() {
    let hart = cpu_id();
    write(SENABLE+SENABLE_HART*hart, (1<<UART0_IRQ)|(1<<VIRTIO0_IRQ));
    write(SPRIORITY+SPRIORITY_HART*hart, 0);
}

// !!! pay attention to uint32
pub unsafe fn plic_claim() -> u32 {
    let hart = cpu_id();
    read(SCLAIM+SCLAIM_HART*hart)
}

// !!! pay attention to uint32
pub unsafe fn plic_complete(irq: u32) {
    let hart = cpu_id();
    write(SCLAIM+SCLAIM_HART*hart, irq);
}

#[inline]
fn read(offset: usize) -> u32 {
    unsafe {
        let src = (PLIC + offset) as *const u32;
        ptr::read_volatile(src)
    }
}

#[inline]
fn write(offset: usize, value: u32) {
    unsafe {
        let dst = (PLIC + offset) as *mut u32;
        ptr::write_volatile(dst, value);
    }
}
