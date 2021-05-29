//! Physical memory layout
//!
//! qemu -machine virt is set up like this,
//! based on qemu's hw/riscv/virt.c:
//!
//! 00001000 -- boot ROM, provided by qemu
//! 02000000 -- CLINT
//! 0C000000 -- PLIC
//! 10000000 -- uart0
//! 10001000 -- virtio disk
//! 80000000 -- boot ROM jumps here in machine mode
//!             -kernel loads the kernel here
//! unused RAM after 80000000.
//!
//! the kernel uses physical memory thus:
//! 80000000 -- entry.S, then kernel text and data
//! end -- start of kernel page allocation area
//! PHYSTOP -- end RAM used by the kernel

use super::*;

/// where are these come from and refered ?
pub const PGMASK: usize = 0x1FF;
pub const PGMASKLEN: usize = 9;

/// qemu puts UART registers here in physical memory.
pub const UART0: usize = 0x10000000;
pub const UART0_MAP_SIZE: usize = PGSIZE;
pub const UART0_IRQ: usize = 10;

/// virtio mmio interface
pub const VIRTIO0: usize = 0x10001000;
pub const VIRTIO0_IRQ: usize = 1;

/// core local interruptor (CLINT), which contains the timer.
// not implemented
// #define CLINT_MTIMECMP(hartid) (CLINT + 0x4000 + 8*(hartid))
// #define CLINT_MTIME (CLINT + 0xBFF8) // cycles since boot.
pub const CLINT: usize = 0x2000000;
pub const CLINT_MAP_SIZE: usize = 0x10000;
pub const CLINT_MTIMECMP: usize = CLINT + 0x4000;
pub const CLINT_MTIME: usize = CLINT + 0xbff8;

/// qemu puts platform-level interrupt controller (PLIC) here.
// not implemented
// #define PLIC_MENABLE(hart) (PLIC + 0x2000 + (hart)*0x100)
// #define PLIC_SENABLE(hart) (PLIC + 0x2080 + (hart)*0x100)
// #define PLIC_MPRIORITY(hart) (PLIC + 0x200000 + (hart)*0x2000)
// #define PLIC_SPRIORITY(hart) (PLIC + 0x201000 + (hart)*0x2000)
// #define PLIC_MCLAIM(hart) (PLIC + 0x200004 + (hart)*0x2000)
// #define PLIC_SCLAIM(hart) (PLIC + 0x201004 + (hart)*0x2000)
pub const PLIC: usize = 0x0c000000;
pub const PLIC_PRIORITY: usize = PLIC + 0x0;
pub const PLIC_PENDING: usize = PLIC + 0x1000;
pub const PLIC_SIZE: usize = 0x400000;

/// the kernel expects there to be RAM
/// for use by the kernel and user pages
/// from physical address 0x80000000 to PHYSTOP.
pub const KERNBASE: usize = 0x80000000;
// pub const PHYSTOP: usize = (KERNBASE + 128*1024*1024);
pub const PHYSTOP: usize = 0x86400000;

/// map the trampoline page to the highest address,
/// in both user and kernel space.
pub const TRAMPOLINE: usize = MAXVA - PGSIZE;

/// not implemented here, should be implemented as function
// map kernel stacks beneath the trampoline,
// each surrounded by invalid guard pages.
// #define KSTACK(p) (TRAMPOLINE - ((p)+1)* 2*PGSIZE)

/// User memory layout.
/// Address zero first:
///   text
///   original data and bss
///   fixed-size stack
///   expandable heap
///   ...
///   TRAPFRAME (p->trapframe, used by the trampoline)
///   TRAMPOLINE (the same page as in the kernel)
pub const TRAPFRAME: usize = TRAMPOLINE - PGSIZE;