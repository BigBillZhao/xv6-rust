pub mod startup;
pub mod sstatus;
pub mod clint;
pub mod mie;
pub mod mstatus;
pub mod sip;

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

/// r/w satp
/// supervisor address translation and protection;
/// holds the address of the page table.
/// used in vm.rs
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

/// stvec
pub mod stvec {
    pub unsafe fn write(stvec: usize) {
        llvm_asm!("csrw stvec, $0"::"r"(stvec)::"volatile");
    }
}
pub mod scause{
    #[inline]
    pub unsafe fn read() -> usize {
        let ret: usize;
        unsafe {llvm_asm!("csrr $0, scause":"=r"(ret):::"volatile");}
        ret
    }

}
/// sepc
/// machine exception program counter, holds the
/// instruction address to which a return from
/// exception will go.(from xv6-riscv)
pub mod sepc {
    pub fn read() -> usize {
        let ret: usize;
        unsafe {llvm_asm!("csrr $0, sepc":"=r"(ret):::"volatile");}
        ret
    }

    pub fn write(sepc: usize) {
        unsafe {llvm_asm!("csrw sepc, $0"::"r"(sepc)::"volatile");}
}
}
/// mscratch
pub mod mscratch {
    pub unsafe fn write(mscratch: usize) {
        llvm_asm!("csrw mscratch, $0"::"r"(mscratch)::"volatile");
    }
}

/// mtvec
pub mod mtvec {
    pub unsafe fn write(mtvec: usize) {
        llvm_asm!("csrw mtvec, $0"::"r"(mtvec)::"volatile");
    }
}