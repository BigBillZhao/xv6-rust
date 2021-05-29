pub mod kalloc;
pub mod addr;
pub mod pagetable;

use pagetable::PageTable;
use addr::{PteFlag, PTE, PhysAddr, VirtualAddr, Addr};

use array_macro::array;
use alloc::boxed::Box;

use crate::register::satp;
use crate::consts::{PGSIZE, PGSHIFT, SV39FLAGLEN, SATP_SV39, PGMASKLEN, PGMASK};
use crate::consts::{UART0, VIRTIO0, PLIC, PLIC_SIZE, KERNBASE, PHYSTOP, TRAMPOLINE};

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
    println!("TODO: proc_mapstack");
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
    // 0 as * mut PTE // NULL
}


