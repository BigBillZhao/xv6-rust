use super::*;

#[repr(C, align(4096))]
pub struct PageTable {
    data: [PTE; 512],
}

impl PageTable {
    pub const fn new() -> PageTable {
        PageTable {
            data: array![_ => PTE::empty(); 512],
        }
    }
    pub unsafe fn map_pages(
        &mut self, va: VirtualAddr, pa: PhysAddr, size: usize, perm: PteFlag
    ) -> Result<(), &'static str> {
        if ! pa.is_aligned() {
            panic!();
        }
        let mut va_iter: usize = va.round_down();
        let mut pa_iter: PhysAddr = pa; 
        let addr_end: usize = va.add(size).round_up();
        println!("    map_pages, va = {:#x}, end = {:#x}", va_iter, addr_end);
        while va_iter != addr_end {
            let ppte: * mut PTE = walk_alloc(self, VirtualAddr::from(va_iter));
            if (*ppte).is_valid() {
                panic!();
            }
            *ppte = PTE::from_pa(pa_iter, perm | PteFlag::V);
            va_iter += PGSIZE;
            pa_iter = pa_iter.add(PGSIZE); 
        }
        Ok(())
    }
    pub unsafe fn recursive_free(&mut self) {
        for i in (0..512) {
            let pte = self.data[i];
            if pte.is_valid() && (
                pte.get() & (PteFlag::R.bits()|PteFlag::W.bits()|PteFlag::X.bits()) == 0
            ) {
                let pchild: *mut PageTable = pte.to_pagetable();
                (*pchild).recursive_free();
                self.data[i].set(0);
            } else if pte.is_valid() {
                panic!();
            }
        }
        let ppa = self as * mut PageTable;
        drop(Box::from_raw(ppa));
    }
    pub fn as_satp(&self) -> usize {
        SATP_SV39 | ((self.data.as_ptr() as usize) >> PGSHIFT)
    }
}