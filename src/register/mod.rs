pub mod startup;
pub mod mstatus{
    pub unsafe fn read() -> usize {
        let ret: usize;
        llvm_asm!("csrr $0, mstatus":"=r"(ret):::"volatile");
        ret
    }
    pub unsafe fn write(x: usize) {
        llvm_asm!("csrw mstatus, $0"::"r"(x)::"volatile");
    }
}
/// mepc
pub mod mepc {
    pub unsafe fn write(mepc: usize) {
        llvm_asm!("csrw mepc, $0"::"r"(mepc)::"volatile");
    }
}

/// medeleg
pub mod medeleg {
    pub unsafe fn write(medeleg: usize) {
        llvm_asm!("csrw medeleg, $0"::"r"(medeleg)::"volatile");
    }
}

/// mideleg
pub mod mideleg {
    pub unsafe fn write(mideleg: usize) {
        llvm_asm!("csrw mideleg, $0"::"r"(mideleg)::"volatile");
    }
}
pub mod satp{
    //! satp register
    pub fn read() -> usize {
        let ret;
        unsafe {
            llvm_asm!("csrr $0, satp":"=r"(ret):::"volatile");
        }
        ret
    }
    pub fn write(satp: usize) {
        unsafe {
            llvm_asm!("csrw satp, $0"::"r"(satp)::"volatile");
        }
    }

}
pub mod sie{
    //! sie register

    const SSIE: usize = 1 << 1; // software
    const STIE: usize = 1 << 5; // timer
    const SEIE: usize = 1 << 9; // external

    pub unsafe fn read() -> usize {
        let ret: usize;
        llvm_asm!("csrr $0, sie":"=r"(ret):::"volatile");
        ret
    }
    pub unsafe fn write(x: usize) {
        llvm_asm!("csrw sie, $0"::"r"(x)::"volatile");
    }

    /// enable all software interrupts
    /// still need to set SIE bit in sstatus
    pub unsafe fn intr_on() {
        let mut sie = read();
        sie |= SSIE | STIE | SEIE;
        write(sie);
    }


}
/// mhartid
pub mod mhartid {
    pub unsafe fn read() -> usize {
        let ret: usize;
        llvm_asm!("csrr $0, mhartid":"=r"(ret):::"volatile");
        ret
    }
}
/// tp
pub mod tp {
    pub unsafe fn read() -> usize {
        let ret: usize;
        llvm_asm!("mv $0, tp":"=r"(ret):::"volatile");
        ret
    }

    pub unsafe fn write(tp: usize) {
        llvm_asm!("mv tp, $0"::"r"(tp)::"volatile");
    }
}