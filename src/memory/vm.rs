use crate::consts::{UART0, VIRTIO0, PLIC, KERNBASE, PHYSTOP, TRAMPOLINE};
use crate::consts::{PGSIZE, PGSHIFT};
use crate::consts::{PTE_V, PTE_R, PTE_W, PTE_X, PTE_U};
use crate::consts::{PXMASK, MAXVA, SATP_SV39};
use crate::consts::USIZELEN;

use crate::register;
use super::*;
use crate::proc::proc;
use core::ptr;
use crate::memory::kalloc::{kfree,kalloc,round_up};

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
            // println!("in walk, calls kalloc, returns: {:#x}", pagetable);
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
pub unsafe fn kvmmap(kpgtbl:usize, va: usize, pa: usize, size: usize, perm: usize){
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
    // let flag = false;
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
unsafe fn uvmunmap(pagetable: usize, va: usize, npages:usize, do_free:bool){
    if va % PGSIZE != 0 {
        panic!("uvmunmap: not aligned");
    }

    let mut a = va;
    while a < va + npages * PGSIZE {
        let pte = walk(pagetable, a, false);
        if pte == 0 {
            panic!("uvmunmap : walk");
        }
        let pte_value = unsafe{
            ptr::read_volatile(pte as * const usize)
        };
        //  check mapped
        //1L <<0 :                  PTE_V
        //(pte_value) & 0x3FF :     PTE_FLAGS
        //(((pte_value) >> 10) << 12):    PTE2PA
        if (pte_value & PTE_V)==0 {
            panic!("uvmunmap: not mapped")
        }
        //  check leaf
        if PTE_FLAGS(pte_value)==0 {
            panic!("uvmunmap: not a leaf");
        }
        if do_free {
            let pa=PTE2PA(pte_value);
            unsafe{
                kfree(pa)
            };
        }
        unsafe{
            ptr::write_volatile(pte as * mut usize, 0);
        }
        a= a+ PGSIZE;
    }
}

/// `kalloc` a page and map it by virtual address `0`
fn uvmcreate() -> usize{
    let pagetable = unsafe{
        kalloc()
    };
    if pagetable == 0 {
        return 0;
    }
    // there can add a memset to make things in this allocated address to 0
    //not very neccessary
    return pagetable;
}

/// copy the user code from src to the physical address, then map it for ready to exec
/// src is the address of the user code(phsical)
/// sz is the user code size
unsafe fn uvminit(pagetable: usize, src: usize, sz: usize){
    if sz >= PGSIZE {
        panic!("inituvm: more than a page");
    }
    let mem = unsafe{
        kalloc()
    };
    // can choose to add a memset here
    //PTE_W|PTE_R|PTE_X|PTE_U
    //(1L << 2)|(1L << 1)|(1L << 3)|(1L << 4)
    mappages(pagetable, 0, PGSIZE, mem,PTE_W|PTE_R|PTE_X|PTE_U);
    //use memmove to move the src code to the address
    //memmove(mem, src, sz);
    //pub unsafe fn copy<T>(src: *const T, dst: *mut T, count: usize)
    unsafe{
        ptr::copy(src as * const usize, mem as * mut usize, sz);
    };
}

/// quote:
// Allocate PTEs and physical memory to grow process from oldsz to
// newsz, which need not be page aligned.  Returns new size or 0 on error.
/// may be return a bool on success is better (result)
pub unsafe fn uvmalloc(pagetable: usize, oldsz: usize, newsz: usize) -> usize{

    if newsz < oldsz{
        return oldsz;
    }
    
    let oldsz = round_up(oldsz);
    let mut a = oldsz;
    while a < newsz {
        let mem = unsafe{
            kalloc()
        };
        if mem == 0{
            uvmdealloc(pagetable, a, oldsz);
            return 0;
        }
        // can add a memset here
        //PTE_W|PTE_R|PTE_X|PTE_U
        //(1L << 2)|(1L << 1)|(1L << 3)|(1L << 4)
        if mappages(pagetable, a, PGSIZE, mem, PTE_W|PTE_R|PTE_X|PTE_U){
            unsafe{
                kfree(mem);
            }
            uvmdealloc(pagetable, a, oldsz);
            return 0;
        }
        a += PGSIZE;
    }
    return newsz;
}

/// the inverse operation of uvmalloc
pub unsafe fn uvmdealloc(pagetable: usize, oldsz: usize, newsz: usize) -> usize{
    if newsz >= oldsz {
        return oldsz;
    }
    let round_newsz = round_up(newsz);
    let round_oldsz = round_up(oldsz);
    if round_newsz < round_oldsz {
        let npages = (round_oldsz - round_newsz)/ PGSIZE;
        uvmunmap(pagetable, round_newsz, npages, true);
    }
    return newsz;
}

/// quote:
// Recursively free page-table pages.
// All leaf mappings must already have been removed.
fn freewalk(pagetable: usize){

}

/// a warpper for `uvmunmap`, then calls `freewalk`(one user process must call it to free all memory)
/// user process has the illusion of continues memory space
unsafe fn uvmfree(pagetable: usize, sz: usize){
    if sz > 0{
        uvmunmap(pagetable, 0, round_up(sz) / PGSIZE, true);
    }
    freewalk(pagetable);
}

/// old and new are `pagetable_t' in c, thus usize for address here
/// copy the physical memory of a parent process to a child process in fork
/// copy src start from addr. `0` by `sz` length and dst. start at `0` as well
/// will have return true if success, return false if have error
pub unsafe fn uvmcopy(old: usize, new: usize, sz: usize) -> bool {
    let mut i = 0;
    while i < sz{
        let pte = walk(old, i, false);
        if pte == 0{
            panic!("uvmcopy: pte should exist");
        }
        let pte_value = unsafe{
            ptr::read_volatile(pte as * const usize)
        };
        //1L <<0 : PTE_V
        if (pte_value & PTE_V) == 0{
            panic!("uvmcopy: page not present");
        }
        //(((pte_value) >> 10) << 12):    PTE2PA
        let pa = PTE2PA(pte_value);
        //(pte_value) & 0x3FF :     PTE_FLAGS
        let flags = PTE_FLAGS(pte_value);
        let mem = unsafe{
            kalloc()
        };
        if mem == 0{
            uvmunmap(new, 0, i/PGSIZE,true);
            //have error not panic
            return false;
        }
        //pub unsafe fn copy<T>(src: *const T, dst: *mut T, count: usize)
        unsafe{
            ptr::copy(pa as * const usize, mem as * mut usize, sz);
        };
        if mappages(new, i, PGSIZE, mem, flags){
            unsafe{
                kfree(mem)
            };
            //have error not panic
            uvmunmap(new, 0, i/PGSIZE,true);
            return false;
        }
        i = i + PGSIZE;
    }
    return true;
}

/// quote:
// mark a PTE invalid for user access.
// used by exec for the user stack guard page.
unsafe fn uvmclear(pagetable: usize, va: usize){
    let pte = walk(pagetable, va, false);
    if pte == 0{
        panic!("uvmclear");
    }
    let pte_value = unsafe{
        ptr::read_volatile(pte as * const usize)
    };
    unsafe{
        // PTE_U
        //*pte &= ~PTE_U;
        ptr::write_volatile(pte as * mut usize,pte_value &!PTE_U);
    }
    
}

///quote:
// Copy from kernel to user.
// Copy len bytes from src to virtual address dstva in a given page table.
// Return true on success, false on error.
unsafe fn copyout(pagetable: usize, dstva: usize, src: usize, len: usize)-> bool{
    let mut len = len;
    let mut src = src;
    let mut dstva = dstva;
    while len > 0{
        let va0 = round_up(dstva);
        let pa0 = walkaddr(pagetable, va0);
        if pa0 == 0{
            return false;
        }
        let n = if PGSIZE - (dstva - va0) > len{
            len
        }else{
            PGSIZE - (dstva - va0)
        };
        //mem move
        unsafe{
            ptr::copy(src as * const usize, (pa0 + (dstva - va0)) as * mut usize, n);
        };
        len = len - n;
        src = src + n;
        dstva = va0 + PGSIZE;
    }
    return true;
}

///quote:
// Copy from user to kernel.
// Copy len bytes to dst from virtual address srcva in a given page table.
// Return true on success, false on error.
unsafe fn copyin(pagetable: usize, dst: usize, srcva: usize, len: usize)-> bool{
    let mut len = len;
    let mut dst = dst;
    let mut srcva = srcva;
    while len > 0{
        let va0 = round_up(srcva);
        let pa0 = walkaddr(pagetable, va0);
        if pa0 == 0{
            return false;
        }
        let n = if PGSIZE - (srcva - va0) > len{
            len
        }else{
            PGSIZE - (srcva - va0)
        };
        //mem move
        unsafe{
            ptr::copy((pa0 + (srcva - va0)) as * const usize, dst as * mut usize, n);
        };
        len = len - n;
        dst = dst + n;
        srcva = va0 + PGSIZE;
    }
    return true;
}

///quote:
// Copy a null-terminated string from user to kernel.
// Copy bytes to dst from virtual address srcva in a given page table,
// until a '\0', or max.
// Return true on success, false on error.
//
//this function may have problem, and the xv6-rust implemet it in pagetable.rs
//it have big difference with this implementation
unsafe fn copyinstr(pagetable: usize, dst: usize, srcva: usize, max: usize)-> bool{
    let mut got_null = false;
    let mut max = max;
    let mut srcva =srcva;
    let mut dst = dst;
    while !got_null && max > 0{
        let va0 = round_up(srcva);
        let pa0 = walkaddr(pagetable, va0);
        if pa0 == 0{
            return false;
        }
        let mut n = if PGSIZE - (srcva - va0) > max{
            max
        }else{
            PGSIZE - (srcva - va0)
        };
        
        //str operation
        let mut p = pa0 + (srcva - va0);
        while n > 0{
            let p_value = unsafe{
                ptr::read_volatile(p as * const char)
            };
            if p_value == '\0'{
                unsafe{
                    ptr::write_volatile(dst as * mut char,'\0');
                }
                got_null = true;
                break;
            }else{
                unsafe{
                    ptr::write_volatile(dst as * mut char,p_value);
                }
            }
            n = n - 1;
            max = max - 1;
            p = p + 1;
            dst = dst + 1;
        }
        
        srcva = va0 + PGSIZE;
    }
    if got_null{
        return true;
    }else{
        return false;
    }
    
}


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



