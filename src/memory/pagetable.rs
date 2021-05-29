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
        if pa.get_self() % PGSIZE != 0 {
            panic!();
        }
        let mut va_iter: usize = va.round_down();
        let mut pa_iter: PhysAddr = pa; 
        let addr_end: usize = va.add(size).round_up();
        println!("map_pages, va = {:#x}, end = {:#x}", va_iter, addr_end);
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
    pub fn as_satp(&self) -> usize {
        SATP_SV39 | ((self.data.as_ptr() as usize) >> PGSHIFT)
    }
}