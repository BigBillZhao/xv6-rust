pub mod kalloc;

use array_macro::array;
use alloc::boxed::Box;
use core::ptr;

use crate::register::satp;
use crate::consts::{PGSIZE, PGSHIFT, SV39FLAGLEN, SATP_SV39, PGMASKLEN, PGMASK};
use crate::consts::{UART0, VIRTIO0, PLIC, KERNBASE, PHYSTOP, TRAMPOLINE};

static mut kernel_pagetable: PageTable = PageTable::new();

pub unsafe fn kvminit() {
    // uart
    kernel_pagetable.map_pages(
        VirtualAddr(UART0),
        PhysAddr(UART0),
        PGSIZE,
        PteFlag::R | PteFlag::W
    );
    println!("uart direct map finished");
    // virtio mmio disk
    kernel_pagetable.map_pages(
        VirtualAddr(VIRTIO0),
        PhysAddr(VIRTIO0),
        PGSIZE,
        PteFlag::R | PteFlag::W
    );
    // PLIC
    kernel_pagetable.map_pages(
        VirtualAddr(PLIC),
        PhysAddr(PLIC),
        0x400000,
        PteFlag::R | PteFlag::W
    );
    extern "C" {
        fn etext();
    }
    let etext = etext as usize;
    // kernel text executable and read-only
    kernel_pagetable.map_pages(
        VirtualAddr(KERNBASE),
        PhysAddr(KERNBASE),
        etext - KERNBASE,
        PteFlag::R | PteFlag::X
    );
    println!("finish map kernel text exec and RO data");
    // kernel data and useable phy RAM
    kernel_pagetable.map_pages(
        VirtualAddr(etext),
        PhysAddr(etext),
        PHYSTOP - etext,
        PteFlag::R | PteFlag::W
    );
    extern "C" {
        fn trampoline();
    }
    let trampoline = trampoline as usize;
    // trampoline
    kernel_pagetable.map_pages(
        VirtualAddr(TRAMPOLINE),
        PhysAddr(trampoline),
        PGSIZE,
        PteFlag::R | PteFlag::X
    );
    println!("TODO: proc_mapstack");
}

pub unsafe fn kvminithart() {
    println!("kernal pagetable is {:#x}", kernel_pagetable.as_satp());
    satp::write(kernel_pagetable.as_satp());
    println!("satp write result is {:#x}", satp::read());
    llvm_asm!("sfence.vma zero, zero"::::"volatile");
}

bitflags! {
    pub struct PteFlag: usize {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct PTE(usize);

impl PTE {
    fn from_pa(pa: PhysAddr, perm: PteFlag) -> PTE {
        PTE((pa.get_self() >> 12) << 10 | perm.bits())
    }

    fn to_pa(&self) -> PhysAddr {
        PhysAddr((self.0 >> SV39FLAGLEN) << PGSHIFT)
    }

    fn to_pagetable(&self) -> * mut PageTable {
        ((self.0 >> SV39FLAGLEN) << PGSHIFT) as * mut PageTable
    }

    fn is_valid(&self) -> bool {
        (self.0 & PteFlag::V.bits()) > 0
    }

}

trait Addr {
    fn get_self(&self) -> &usize;
    fn get_mut_self(&mut self) -> &mut usize;

    fn round_up(&self) -> usize {
        (self.get_self() + PGSIZE - 1) & !(PGSIZE - 1)
    }
    fn round_down(&self) -> usize {
        self.get_self() & !(PGSIZE - 1)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PhysAddr(usize);
impl Addr for PhysAddr {
    fn get_self(&self) -> &usize {
        &self.0
    }

    fn get_mut_self(&mut self) -> &mut usize {
        &mut self.0
    }
}
impl PhysAddr {
    pub fn add(&self, another: usize) -> PhysAddr {
        PhysAddr(self.0 + another)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct VirtualAddr(usize);
impl Addr for VirtualAddr {
    fn get_self(&self) -> &usize {
        &self.0
    }

    fn get_mut_self(&mut self) -> &mut usize {
        &mut self.0
    }
}
impl VirtualAddr{
    pub unsafe fn to_pa(&self, pagetable: &mut PageTable) -> Option<PhysAddr> {
        let mut pgt = pagetable as * mut PageTable;
        for level in (0..3).rev() {
            let pte = *self.resolve_level(pgt, level);
            if ! pte.is_valid() {
                return None
            }
            if level == 0 {
                return Some(pte.to_pa())
            }
            pgt = pte.to_pagetable();
        }
        None
    }

    pub fn resolve_level(&self, pagetable: * const PageTable, level: usize) -> * mut PTE {
        unsafe {
            // let pte = (*pagetable).data[(self.0 >> (PGSHIFT + level * PGMASKLEN)) & PGMASK];
            // println!("in resolve level, pte = {}", pte.0);
            // pte as * mut PTE
            let pagetable = pagetable as usize;
            (pagetable + ((self.0 >> (PGSHIFT + level * PGMASKLEN)) & PGMASK) * 8) as * mut PTE
        }
    }

    pub fn add(&self, another: usize) -> VirtualAddr {
        VirtualAddr(self.0 + another)
    }
}

#[repr(C, align(4096))]
struct PageTable {
    data: [PTE; 512],
}

impl PageTable {
    const fn new() -> PageTable {
        PageTable {
            data: array![_ => PTE(0); 512],
        }
    }
    
    unsafe fn map_pages(
        &mut self, va: VirtualAddr, pa: PhysAddr, size: usize, perm: PteFlag
    ) -> Result<(), &'static str> {
        if pa.get_self() % PGSIZE != 0 {
            panic!();
        }
        let mut va_iter: usize = va.round_down();
        let mut pa_iter: PhysAddr = pa; 
        let addr_end: usize = va.add(size).round_up();
        println!("map_pages, va = {:#x}, end = {:#x}", va_iter, addr_end);
        while va_iter != addr_end {
            let ppte: * mut PTE = walk_alloc(self, VirtualAddr(va_iter));
            // println!("ppte should be {:p}", ppte);
            // println!("*ppte is {:#x}", (*ppte).0);
            // println!("can exit walk_alloc");
            if (*ppte).is_valid() {
                println!("in is valid, next to panic: the addr is {:p}", ppte);
                panic!("ppte allocated before alloc {:#x}, the pa is {:#x}", (*ppte).0, pa_iter.get_self());
            }
            *ppte = PTE::from_pa(pa_iter, perm | PteFlag::V);
            if (*ppte).0 == 0x20007c01 {
                // println!("the addr is {:p}", ppte);
                // panic!("ppte allocated before alloc {:#x}, the pa is {:#x}", (*ppte).0, pa_iter.get_self());
            }
            va_iter += PGSIZE;
            //println!("va_iter = {:#x}", va_iter);
            pa_iter = pa_iter.add(PGSIZE); 
        }
        Ok(())
    }

    fn as_satp(&self) -> usize {
        SATP_SV39 | ((self.data.as_ptr() as usize) >> PGSHIFT)
    }
}

unsafe fn walk_alloc(pagetable: &mut PageTable, va: VirtualAddr) -> * mut PTE {
    let mut pgt = pagetable as * mut PageTable;
    //println!("how are you");
    for level in (0..3).rev() {
        //println!("hello? level = {}", level);
        let ppte: * mut PTE  = va.resolve_level(pgt, level);
        if level == 0 {
            // println!("walk alloc will return {:p}", ppte);
            return ppte
        }
        if ! (*ppte).is_valid() {
            //println!("Before box");
            let sub_pgt: Box<PageTable> = unsafe { Box::new_zeroed().assume_init() };
            pgt = Box::into_raw(sub_pgt);
            println!("pgt is at {:p} after alloc, level = {}", pgt, level);
            *ppte = PTE::from_pa(PhysAddr(pgt as usize), PteFlag::V);
            let mut cnt = 0;
            for iter in (*(*ppte).to_pagetable()).data.iter() {
                cnt += 1;
                if iter.0 != 0 {
                    panic!("iter is {:p}", &iter);
                }
            }
            // println!("cnt is {}", cnt);
            //println!("After box");
        } else {
            //println!("no valid here");
            pgt = (*ppte).to_pagetable();
        }
    }
    panic!();
    0 as * mut PTE // NULL
}


