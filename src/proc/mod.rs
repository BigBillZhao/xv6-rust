pub mod proc;

/// Allocate a page for each process's kernel stack.
/// Map it high in memory, followed by an invalid
/// guard page.
pub fn proc_mapstacks(kpgtbl: usize){

}