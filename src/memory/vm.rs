
/// direct map the kernel memory
/// the return type is the address, used as a pointer
fn kvmmake() -> usize{

}

/// calal kvmmake and give the value to a global variable kernel_pagetable
fn kvminit (){

}

/// flush the TLB
fn kvminithart(){

}

/// the 'core' of the `vm`
/// the return type, and argument pagetable, va are the addresses, used as a pointer
fn walk(pagetable: usize, va: usize) -> usize{

}

/// A warrper for walk
/// the return type, and argument pagetable, va are the addresses, used as a pointer
fn walkaddr(pagetable:usize, va:usize) -> usize{

}

/// a wrapper for the mappages, specifically for kernel page table `kpgtbl` parameter
fn kvmmap(kpgtbl:usize, va: usize, size: usize, pa: usize, perm: u64){

}

/// map the memory from `va` by `size` amount to `pa` and use `perm` as the bit mask
/// use pagetable` as page table
/// return if success
/// pagetable, va, pa are the addresses, used as a pointer
/// pte is of 64 bit long, so is perm
/// size is the difference of pointers
fn mappages(pagetable: usize, va: usize, size: usize, pa: usize, perm: u64) -> bool{

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

}

/// the inverse operation of uvmalloc
fn uvmdealloc(pagetble: usize, oldsz: usize, newsz: usize) -> usize{

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

}

/// quote:
// mark a PTE invalid for user access.
// used by exec for the user stack guard page.
fn uvmclear(pagetable: usize, va: usize){

}

fn copyout()
fn copyin()
fn copyinstr()