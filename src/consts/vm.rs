/// The constants in this file is corresponding to the constants in `riscv.h`

use super::*;

pub const PGSIZE: usize = 4096;
pub const PGSHIFT: usize = 12;

/// PTE bit
pub const PTE_V: usize = 0b1; // valid if set
pub const PTE_R: usize = 0b10;
pub const PTE_W: usize = 0b100;
pub const PTE_X: usize = 0b1000;
pub const PTE_U: usize = 0b10000; // user can access if set

/// extract the three 9-bit page table indices from a virtual address.
pub const PXMASK: usize = 0x1FF;

/// one beyond the highest possible virtual address.
/// MAXVA is actually one bit less than the max allowed by
/// Sv39, to avoid having to sign-extend virtual addresses
/// that have the high bit set.
// MAXVA = (1L << (9 + 9 + 9 + 12 - 1)) = 1L << (9 + 9 + 9 + 9 + 2)
pub const MAXVA: usize = 1 << (9 + 9 + 9 + 12 - 1);
// pub const MAXVA: usize = 0b1_000000000_000000000_000000000_000000000_00;

/// the following should be implemented as functions:
// #define PGROUNDUP(sz)  (((sz)+PGSIZE-1) & ~(PGSIZE-1))
// #define PGROUNDDOWN(a) (((a)) & ~(PGSIZE-1))
/// shift a physical address to the right place for a PTE.
// #define PA2PTE(pa) ((((uint64)pa) >> 12) << 10)
// #define PTE2PA(pte) (((pte) >> 10) << 12)
// #define PTE_FLAGS(pte) ((pte) & 0x3FF)
/// extract the three 9-bit page table indices from a virtual address.
// #define PXSHIFT(level)  (PGSHIFT+(9*(level)))
// #define PX(level, va) ((((uint64) (va)) >> PXSHIFT(level)) & PXMASK)

/// use riscv's sv39 page table scheme.
// SATP_SV39 = (8L << 60)
pub const SATP_SV39: usize = 8 << 60;
// pub const SATP_SV39: usize = 0b1000_0000000000_0000000000_0000000000_0000000000_0000000000_0000000000;

pub const SV39FLAGLEN: usize = 10;
