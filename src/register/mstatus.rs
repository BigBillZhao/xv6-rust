//! mstatus register

use bit_field::BitField;

#[inline]
pub unsafe fn read() -> usize {
    let ret: usize;
    llvm_asm!("csrr $0, mstatus":"=r"(ret):::"volatile");
    ret
}

#[inline]
pub unsafe fn write(x: usize) {
    llvm_asm!("csrw mstatus, $0"::"r"(x)::"volatile");
}

/// Machine Previous Privilege Mode
pub enum MPP {
    User = 0,
    Supervisor = 1,
    Machine = 3,
}

/// set MPP field
pub unsafe fn set_mpp(mpp: MPP) {
    let mut mstatus = read();
    mstatus.set_bits(11..13, mpp as usize);
    write(mstatus);
}

/// set MIE field
pub unsafe fn set_mie() {
    let mut mstatus = read();
    mstatus.set_bit(3, true);
    write(mstatus);
}
