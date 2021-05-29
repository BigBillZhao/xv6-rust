pub mod kalloc;
pub mod addr;
pub mod pagetable;

use pagetable::PageTable;
use addr::{PteFlag, PTE, PhysAddr, VirtualAddr, Addr};

use array_macro::array;
use alloc::boxed::Box;

use crate::register::satp;
use crate::consts::{PGSIZE, PGSHIFT, SV39FLAGLEN, SATP_SV39, PGMASKLEN, PGMASK};
use crate::consts::{UART0, VIRTIO0, PLIC, PLIC_SIZE, KERNBASE, PHYSTOP, TRAMPOLINE, NPROC};

static mut KERNEL_PAGETABLE: PageTable = PageTable::new();

pub unsafe fn kvminit() {
    // uart
    println!("map uart: ");
    KERNEL_PAGETABLE.map_pages(
        VirtualAddr::from(UART0),
        PhysAddr::from(UART0),
        PGSIZE,
        PteFlag::R | PteFlag::W
    );
    // virtio mmio disk
    println!("map virtio mmio disk: ");
    KERNEL_PAGETABLE.map_pages(
        VirtualAddr::from(VIRTIO0),
        PhysAddr::from(VIRTIO0),
        PGSIZE,
        PteFlag::R | PteFlag::W
    );
    // PLIC
    println!("map plic: ");
    KERNEL_PAGETABLE.map_pages(
        VirtualAddr::from(PLIC),
        PhysAddr::from(PLIC),
        PLIC_SIZE,
        PteFlag::R | PteFlag::W
    );
    extern "C" {
        fn etext();
    }
    let etext = etext as usize;
    // kernel text executable and read-only
    println!("map kernel text executable and read-only: ");
    KERNEL_PAGETABLE.map_pages(
        VirtualAddr::from(KERNBASE),
        PhysAddr::from(KERNBASE),
        etext - KERNBASE,
        PteFlag::R | PteFlag::X
    );
    // kernel data and useable phy RAM
    println!("map kernel data and usable physical RAM: ");
    KERNEL_PAGETABLE.map_pages(
        VirtualAddr::from(etext),
        PhysAddr::from(etext),
        PHYSTOP - etext,
        PteFlag::R | PteFlag::W
    );
    extern "C" {
        fn trampoline();
    }
    let trampoline = trampoline as usize;
    // trampoline
    println!("map trampoline: ");
    KERNEL_PAGETABLE.map_pages(
        VirtualAddr::from(TRAMPOLINE),
        PhysAddr::from(trampoline),
        PGSIZE,
        PteFlag::R | PteFlag::X
    );
    println!("proc_mapstacks: ");
    for i in 0..NPROC {
        let pa: Box<PageTable> = unsafe { Box::new_zeroed().assume_init() };
        KERNEL_PAGETABLE.map_pages(
            VirtualAddr::from(KSTACK(i)),
            PhysAddr::from(Box::into_raw(pa) as usize),
            PGSIZE,
            PteFlag::R | PteFlag::W
        );
    }
}

pub unsafe fn kvminithart() {
    satp::write(KERNEL_PAGETABLE.as_satp());
    llvm_asm!("sfence.vma zero, zero"::::"volatile");
    println!("TLB flushed!");
}

unsafe fn walk_alloc(pagetable: &mut PageTable, va: VirtualAddr) -> * mut PTE {
    let mut pgt = pagetable as * mut PageTable;
    for level in (0..3).rev() {
        let ppte: * mut PTE  = va.resolve_level(pgt, level);
        if level == 0 {
            return ppte
        }
        if ! (*ppte).is_valid() {
            let sub_pgt: Box<PageTable> = unsafe { Box::new_zeroed().assume_init() };
            pgt = Box::into_raw(sub_pgt);
            *ppte = PTE::from_pa(PhysAddr::from(pgt as usize), PteFlag::V);
        } else {
            pgt = (*ppte).to_pagetable();
        }
    }
    panic!();
}

unsafe fn walk_free(pagetable: &mut PageTable, va: VirtualAddr) -> * mut PTE {
    let mut pgt = pagetable as * mut PageTable;
    for level in (0..3).rev() {
        let ppte: * mut PTE  = va.resolve_level(pgt, level);
        if level == 0 {
            return ppte
        }
        if ! (*ppte).is_valid() {
            panic!();
        } else {
            pgt = (*ppte).to_pagetable();
        }
    }
    panic!();
}

pub fn KSTACK(p: usize) -> usize{
    TRAMPOLINE - ((p + 1) * 2 * PGSIZE)
}

unsafe fn uvmunmap(
    pagetable: &mut PageTable, va: VirtualAddr, npages: usize, do_free: bool
) {
    if ! va.is_aligned() {
        panic!();
    }
    let mut va_iter = va;
    let mut cnt = 0;
    while cnt < npages {
        let ppte: * mut PTE = walk_free(pagetable, va_iter);
        if ! (*ppte).is_valid() {
            panic!();
        }
        if do_free {
            let ppa = (*ppte).to_pa().get() as * mut PageTable;
            drop(Box::from_raw(ppa));
        }
        (*ppte).set(0);
        va_iter = va_iter.add(PGSIZE);
        cnt += 1;
    }
}

pub unsafe fn uvmfree(pagetable: &mut PageTable, size: usize) {
    if size > 0 {
        let va = VirtualAddr::from(0);
        uvmunmap(
            pagetable, va, va.add(size).round_up() / PGSIZE, true
        );
    }
    pagetable.recursive_free();
}

pub unsafe fn uvmcreate() -> Box<PageTable> {
    // let pagetable: Box<PageTable> = Box::new_zeroed().assume_init();
    // pagetable
    Box::<PageTable>::new_zeroed().assume_init()
}