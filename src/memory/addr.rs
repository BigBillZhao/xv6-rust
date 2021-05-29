use super::*;

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
pub struct PTE(usize);

impl PTE {
    pub const fn empty() -> PTE {
        PTE(0)
    }
    pub fn from_pa(pa: PhysAddr, perm: PteFlag) -> PTE {
        PTE((pa.get_self() >> 12) << 10 | perm.bits())
    }
    pub fn to_pa(&self) -> PhysAddr {
        PhysAddr((self.0 >> SV39FLAGLEN) << PGSHIFT)
    }
    pub fn to_pagetable(&self) -> * mut PageTable {
        ((self.0 >> SV39FLAGLEN) << PGSHIFT) as * mut PageTable
    }
    pub fn is_valid(&self) -> bool {
        (self.0 & PteFlag::V.bits()) > 0
    }
    pub fn get(&self) -> usize {
        self.0
    }
    pub fn set(&mut self, val: usize) {
        self.0 = val;
    }
}

pub trait Addr {
    fn get_self(&self) -> &usize;
    fn get_mut_self(&mut self) -> &mut usize;

    fn round_up(&self) -> usize {
        (self.get_self() + PGSIZE - 1) & !(PGSIZE - 1)
    }
    fn round_down(&self) -> usize {
        self.get_self() & !(PGSIZE - 1)
    }
    fn is_aligned(&self) -> bool {
        self.get_self() % PGSIZE == 0
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
    pub const fn from(addr: usize) -> PhysAddr {
        PhysAddr(addr)
    }    
    pub fn add(&self, another: usize) -> PhysAddr {
        PhysAddr(self.0 + another)
    }
    pub fn get(&self) -> usize {
        self.0
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
    pub const fn from(addr: usize) -> VirtualAddr {
        VirtualAddr(addr)
    }
    pub fn add(&self, another: usize) -> VirtualAddr {
        VirtualAddr(self.0 + another)
    }
    pub fn set(&mut self, value:usize){
        self.0= value;
    }
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
        let pagetable = pagetable as usize;
        (pagetable + ((self.0 >> (PGSHIFT + level * PGMASKLEN)) & PGMASK) * 8) as * mut PTE
    }
}