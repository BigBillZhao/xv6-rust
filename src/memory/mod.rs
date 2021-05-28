pub mod kalloc;

use array_macro::array;

use crate::register::satp;
use crate::consts::{PGSIZE, PGSHIFT, SV39FLAGLEN, SATP_SV39, PGMASKLEN, PGMASK};

// pub fn kvminit() {
//     println!("kvminit");
// }

// pub fn kvminithart() {
//     println!("kvminithart");
// }

static mut kernel_pagetable: PageTable = PageTable::new();

pub fn kvminit() {
    kvmmake();
}

pub unsafe fn kvminithart() {
    satp::write(kernel_pagetable.as_satp());
    llvm_asm!("sfence.vma zero, zero"::::"volatile");
}

fn kvmmake(){
    println!("kvmmake");
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

    fn to_pagetable(&self) -> * const PageTable {
        ((self.0 >> SV39FLAGLEN) << PGSHIFT) as * const PageTable
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
    pub fn to_pa(&self, pagetable: &PageTable) -> Option<PhysAddr> {
        let pgt = pagetable as * const PageTable;
        for level in (0..3).rev() {
            let pte = self.resolve_level(pgt, level);
            if ! pte.is_valid() {
                return None
            }
            if level == 0 {
                return Some(pte.to_pa())
            }
            let pgt = pte.to_pagetable();
        }
        None
    }

    pub fn resolve_level(&self, pagetable: * const PageTable, level: usize) -> PTE {
        unsafe {
            (*pagetable).data[(self.0 >> (PGSHIFT + level * PGMASKLEN)) & PGMASK]
        }
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

    fn map_pages(
        &self, va: VirtualAddr, pa: PhysAddr, size: usize, perm: PteFlag
    ) -> Result<(), &'static str> {
        let mut addr_iter = va.round_down();
        loop {

        }
    }

    fn as_satp(&self) -> usize {
        SATP_SV39 | ((self.data.as_ptr() as usize) >> PGSHIFT)
    }
}


