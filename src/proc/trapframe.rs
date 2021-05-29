
#[repr(C)]
pub struct TrapFrame {
    /*   0 */kernel_satp: usize,   // kernel page table
    /*   8 */kernel_sp: usize,     // top of process's kernel stack
    /*  16 */kernel_trap: usize,   // usertrap()
    /*  24 */epc: usize,           // saved user program counter
    /*  32 */kernel_hartid: usize, // saved kernel tp
    /*  40 */ra: usize,
    /*  48 */sp: usize,
    /*  56 */gp: usize,
    /*  64 */tp: usize,
    /*  72 */t0: usize,
    /*  80 */t1: usize,
    /*  88 */t2: usize,
    /*  96 */s0: usize,
    /* 104 */s1: usize,
    /* 112 */pub a0: usize,
    /* 120 */a1: usize,
    /* 128 */a2: usize,
    /* 136 */a3: usize,
    /* 144 */a4: usize,
    /* 152 */a5: usize,
    /* 160 */a6: usize,
    /* 168 */a7: usize,
    /* 176 */s2: usize,
    /* 184 */s3: usize,
    /* 192 */s4: usize,
    /* 200 */s5: usize,
    /* 208 */s6: usize,
    /* 216 */s7: usize,
    /* 224 */s8: usize,
    /* 232 */s9: usize,
    /* 240 */s10: usize,
    /* 248 */s11: usize,
    /* 256 */t3: usize,
    /* 264 */t4: usize,
    /* 272 */t5: usize,
    /* 280 */t6: usize,
}
impl TrapFrame{
  pub const fn TrapFrame()->Self{
    Self{
      /*   0 */kernel_satp: 0,   // kernel page table
    /*   8 */kernel_sp: 0,     // top of process's kernel stack
    /*  16 */kernel_trap: 0,   // usertrap()
    /*  24 */epc: 0,           // saved user program counter
    /*  32 */kernel_hartid: 0, // saved kernel tp
    /*  40 */ra: 0,
    /*  48 */sp: 0,
    /*  56 */gp: 0,
    /*  64 */tp: 0,
    /*  72 */t0: 0,
    /*  80 */t1: 0,
    /*  88 */t2: 0,
    /*  96 */s0: 0,
    /* 104 */s1: 0,
    /* 112 */a0: 0,
    /* 120 */a1: 0,
    /* 128 */a2: 0,
    /* 136 */a3: 0,
    /* 144 */a4: 0,
    /* 152 */a5: 0,
    /* 160 */a6: 0,
    /* 168 */a7: 0,
    /* 176 */s2: 0,
    /* 184 */s3: 0,
    /* 192 */s4: 0,
    /* 200 */s5: 0,
    /* 208 */s6: 0,
    /* 216 */s7: 0,
    /* 224 */s8: 0,
    /* 232 */s9: 0,
    /* 240 */s10: 0,
    /* 248 */s11: 0,
    /* 256 */t3: 0,
    /* 264 */t4: 0,
    /* 272 */t5: 0,
    /* 280 */t6: 0,
    }

  }
}
  