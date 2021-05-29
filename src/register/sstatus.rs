const SIE: usize = 1 << 1;  // supervisor interrupt enable
const SPIE: usize = 1 << 5; // supervisor previous interrupt enable
const SPP: usize = 1 << 8;  // previous mode, is from supervisor?


#[inline]
pub fn read() -> usize {
    let ret: usize;
    unsafe {llvm_asm!("csrr $0, sstatus":"=r"(ret):::"volatile");}
    ret
}

#[inline]
pub fn write(x: usize) {
    unsafe {llvm_asm!("csrw sstatus, $0"::"r"(x)::"volatile");}
}

/// set SIE to enable device interrupts
/// still need to set relevant bit in sie register
#[inline]
pub fn intr_on() {
    write(read() | SIE);
}

/// disable device interrupts
#[inline]
pub fn intr_off() {
    write(read() & !SIE);
}

/// are device interrupts enabled?
#[inline]
pub fn intr_get() -> bool {
    let x = read();
    (x & SIE) != 0
}
