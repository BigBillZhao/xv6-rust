use crate::consts::{UART0, VIRTIO0, PLIC, KERNBASE, PHYSTOP, TRAMPOLINE};
use crate::consts::{PGSIZE, PGSHIFT};
use crate::consts::{PTE_V, PTE_R, PTE_W, PTE_X, PTE_U};
use crate::consts::{PXMASK, MAXVA, SATP_SV39};
use crate::consts::USIZELEN;

use crate::register;
use super::*;
use crate::proc;
use core::ptr;

static mut kernel_pagetable: usize = 0;

/// direct map the kernel memory
/// the return type is the address, used as a pointer
unsafe fn kvmmake() -> usize{
    extern "C" {
        fn etext(); // kernel.ld sets this to end of kernel code.
    }
    let etext = etext as usize;

    extern "C" {
        fn trampoline(); // trampoline.S sets the global entry.
    }
    let trampoline = trampoline as usize;

    let kpgtbl: usize = kalloc::kalloc();
    println!("kpgtbl: {:#x}", kpgtbl);

    // uart reg
    kvmmap(kpgtbl, UART0, UART0, PGSIZE, PTE_R | PTE_W);

    // virtio mmio disk interface
    kvmmap(kpgtbl, VIRTIO0, VIRTIO0, PGSIZE, PTE_R | PTE_W);

    // PLIC
    kvmmap(kpgtbl, PLIC, PLIC, 0x400000, PTE_R | PTE_W);

    // map kernel text executable and read-only.
    kvmmap(kpgtbl, KERNBASE, KERNBASE, etext - KERNBASE, PTE_R | PTE_X);

    // kvmmap(kpgtbl, 0x87ba2000, 0x87ba2000, 0x87fff000 - 0x87ba2000, PTE_R | PTE_W);

    // map kernel data and the physical RAM we'll make use of.
    println!("kvmmap etext addr: {:#x}, size: {:#x}", etext, PHYSTOP - etext);
    kvmmap(kpgtbl, etext, etext, PHYSTOP - etext - 0x44f000, PTE_R | PTE_W);

    // map the trampoline for trap entry/exit to
    // the highest virtual address in the kernel.
    println!("kvmmap trampoline: {:#x}, {:#x}", TRAMPOLINE, trampoline);
    kvmmap(kpgtbl, TRAMPOLINE, trampoline, PGSIZE, PTE_R | PTE_X);
    // map kernel stacks
    proc::proc_mapstacks(kpgtbl);

    kpgtbl
}

/// calal kvmmake and give the value to a global variable kernel_pagetable
pub unsafe fn kvminit(){
    kernel_pagetable = kvmmake();
}

/// flush the TLB
pub unsafe fn kvminithart(){
    register::satp::write(MAKE_SATP(kernel_pagetable));
    println!("goes here");
    // llvm_asm!("sfence.vma zero, zero"::::"volatile");
    register::sfence_vma();
}

/// the 'core' of the `vm`
/// the return type, and argument pagetable, va are the addresses, used as a pointer
unsafe fn walk(pagetable: usize, va: usize, alloc: bool) -> usize{
    let mut pagetable = pagetable;
    if va > MAXVA {
        panic!();
    }
    for level in (0..3).rev() {
        let p_pte = pagetable + PX(level, va) * USIZELEN;
        let pte: usize = unsafe { ptr::read_volatile(p_pte as * const usize) };
        if pte & PTE_V != 0 {
            pagetable = PTE2PA(pte);
        } else {
            if alloc != true {
                panic!();
                return 0;
            }
            pagetable = kalloc::kalloc();
            println!("in walk, calls kalloc, returns: {:#x}", pagetable);
            if pagetable == 0 {
                panic!();
                return 0;
            }
            let new_entry: usize = PA2PTE(pagetable) | PTE_V;
            unsafe { ptr:: write_volatile(p_pte as * mut usize, new_entry); }
        }
    }
    pagetable + PX(0, va) * USIZELEN
}

/// A warrper for walk
/// the return type, and argument pagetable, va are the addresses, used as a pointer
unsafe fn walkaddr(pagetable: usize, va: usize) -> usize{
    if va > MAXVA {
        return 0;
    }
    let p_pte = walk(pagetable, va, false);
    if p_pte == 0 {
        return 0;
    }
    let pte: usize = unsafe { ptr::read_volatile(p_pte as * const usize) };
    if pte & PTE_V == 0 {
        return 0;
    }
    if pte & PTE_U == 0 {
        return 0;
    }
    PTE2PA(pte)
}

/// a wrapper for the mappages, specifically for kernel page table `kpgtbl` parameter
unsafe fn kvmmap(kpgtbl:usize, va: usize, pa: usize, size: usize, perm: usize){
    if mappages(kpgtbl, va, size, pa, perm) == false {
        panic!();
    }
}

/// map the memory from `va` by `size` amount to `pa` and use `perm` as the bit mask
/// use pagetable` as page table
/// return if success
/// pagetable, va, pa are the addresses, used as a pointer
/// pte is of 64 bit long, so is perm
/// size is the difference of pointers
unsafe fn mappages(pagetable: usize, va: usize, size: usize, pa: usize, perm: usize) -> bool{
    
    let flag: bool = {va == 0x80006000};
    let flag = false;
    if flag {println!("print flag set");}
    let mut addr: usize = PGROUNDDOWN(va);
    let last: usize = PGROUNDDOWN(va + size - 1);
    println!("intput addr: {:#x}, last: {:#x}", addr, last);
    let mut pa = pa;
    loop {
        let p_pte: usize = walk(pagetable, addr, true);
        // println!("p_pte = {}", p_pte);
        if p_pte == 0 {
            return false;
        }
        let mut pte: usize = unsafe { ptr::read_volatile(p_pte as * const usize) };
        if pte & PTE_V != 0 {
            panic!();
        }
        pte = PA2PTE(pa) | perm | PTE_V;
        unsafe { ptr:: write_volatile(p_pte as * mut usize, pte); }
        if flag {println!("addr:{:#x}, last:{:#x}", addr, last);}
        if addr == last {
            break;
        }
        addr += PGSIZE;
        pa += PGSIZE;
    }
    // panic!();
    true
}

/// A quotation form the xv6-c version:
// Remove npages of mappings starting from va. va must be
// page-aligned. The mappings must exist.
// Optionally free the physical memory.
/// `npages` is the integer of pages, use usize natually
fn uvmunmap(pagetable: usize, va: usize, npages:usize, do_free:bool){

}

/// `kalloc` a page and map it by virtual address `0`
fn uvmcreate() -> usize{
    0
}

/// copy the user code from src to the physical address, then map it for ready to exec
/// src is the address of the user code(phsical)
/// sz is the user code size
fn uvminit(pagetable: usize, src: usize, sz: usize){

}

/// quote:
// Allocate PTEs and physical memory to grow process from oldsz to
// newsz, which need not be page aligned.  Returns new size or 0 on error.
/// may be return a bool on success is better (result)
fn uvmalloc(pagetble: usize, oldsz: usize, newsz: usize) -> usize{
    0
}

/// the inverse operation of uvmalloc
fn uvmdealloc(pagetble: usize, oldsz: usize, newsz: usize) -> usize{
    0
}

/// quote:
// Recursively free page-table pages.
// All leaf mappings must already have been removed.
fn freewalk(pagetable: usize){

}

/// a warpper for `uvmunmap`, then calls `freewalk`(one user process must call it to free all memory)
/// user process has the illusion of continues memory space
fn uvmfree(pagetable: usize, sz: usize){

}

/// old and new are `pagetable_t' in c, thus usize for address here
/// copy the physical memory of a parent process to a child process in fork
/// copy src start from addr. `0` by `sz` length and dst. start at `0` as well
fn uvmcopy(old: usize, new: usize, sz: usize) -> bool {
    true
}

/// quote:
// mark a PTE invalid for user access.
// used by exec for the user stack guard page.
fn uvmclear(pagetable: usize, va: usize){

}

// fn copyout()
// fn copyin()
// fn copyinstr()

fn MAKE_SATP(pagetable: usize) -> usize {
    SATP_SV39 | (pagetable >> 12)
}

fn PGROUNDUP(size: usize) -> usize {
    (size + PGSIZE - 1) & !(PGSIZE - 1)
}

fn PGROUNDDOWN(addr: usize) -> usize {
    addr & !(PGSIZE - 1)
}

fn PA2PTE(pa: usize) -> usize {
    (pa >> 12) << 10
}

fn PTE2PA(pte: usize) -> usize {
    (pte >> 10) << 12
}

fn PTE_FLAGS(pte: usize) -> usize {
    pte & 0x3FF
}

fn PXSHIFT(level: usize) -> usize {
    PGSHIFT + 9 * level
}

fn PX(level: usize, va: usize) -> usize {
    (va >> PXSHIFT(level)) & PXMASK
}
