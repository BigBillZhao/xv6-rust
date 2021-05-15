use spin::Mutex;
use core::ptr;
use crate::consts::{PGSIZE, PHYSTOP};

struct Kmem{
    freelist: usize,
    END: usize,
}

impl Kmem{
    fn _kfree(&mut self, pa: usize){
        if pa % PGSIZE != 0 {
            panic!();
        }
        if pa < self.END {
            panic!();
        }
        if pa >= PHYSTOP {
            panic!();
        }
        let pa_ptr = pa as *mut usize;
        unsafe{
            ptr::write_volatile(pa_ptr, self.freelist);
        }
        self.freelist = pa;
    }

    fn _kalloc(&mut self) -> usize{
        let p_rtn = self.freelist;
        let pa = p_rtn as * const usize;
        self.freelist = unsafe{
            ptr::read_volatile(pa)
        };
        p_rtn
    }
}

static mut KMEM: Mutex<Kmem> = Mutex::new(Kmem{freelist:0, END:0});

pub unsafe fn kfree(pa: usize){
    KMEM.lock()._kfree(pa);
}

pub unsafe fn kalloc() -> usize {
    KMEM.lock()._kalloc()
}

fn round_up(pa: usize) -> usize {
    if pa % PGSIZE != 0 {
        ((pa / PGSIZE) + 1) * PGSIZE
    } else {
        pa
    }
}

pub unsafe fn kinit(){
    extern "C" {
        fn end();
    }
    let end = end as usize;
    KMEM.lock().END = end as usize;
    let pa_start = KMEM.lock().END;
    let pa_end = PHYSTOP;
    println!("KernelHeap: available physical memory [{:#x}, {:#x})", end, pa_end);
    let pa_start = round_up(pa_start);
    let mut iter = pa_start;
    while iter < pa_end {
        kfree(iter);
        iter = iter + PGSIZE;
    }
}

