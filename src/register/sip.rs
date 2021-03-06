//! Supervisor Interrupt Pending

const SSIP: usize = 1 << 1;

#[inline]
pub unsafe fn read() -> usize {
    let ret: usize;
    llvm_asm!("csrr $0, sip":"=r"(ret):::"volatile");
    ret
}

#[inline]
pub unsafe fn write(x: usize) {
    llvm_asm!("csrw sip, $0"::"r"(x)::"volatile");
}

pub fn clear_ssip() {
    unsafe {
        write(read() & !SSIP);
    }
}
