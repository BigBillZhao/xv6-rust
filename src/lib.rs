#![no_std]
#![feature(llvm_asm)]
#![feature(const_fn)]
#![feature(global_asm)]
#![feature(ptr_internals)]
#![feature(const_fn_union)]
#![feature(slice_ptr_get)]
// #![feature(new_uninit)]
#![feature(alloc_error_handler)]
#![allow(dead_code)]
#![warn(rust_2018_idioms)]


#[macro_use]
extern crate bitflags;
extern crate alloc;

global_asm!(include_str!("asm/entry.S"));
// global_asm!(include_str!("asm/trampoline.S"));
// global_asm!(include_str!("asm/kernelvec.S"));
// global_asm!(include_str!("asm/swtch.S"));

#[macro_use]
mod print;
mod consts;
mod console;
mod register;
mod rmain;
mod start;
mod memory;
mod proc;
