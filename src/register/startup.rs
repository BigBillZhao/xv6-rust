use bit_field::BitField;
use crate::register::mstatus;
use crate::register::medeleg;
use crate::register::mideleg;
use crate::register::mhartid;
use crate::register::tp;
use crate::register::sie;
/// change to supervisor mode
pub unsafe fn mstatus() {
    let mut mstatus_r = mstatus::read();
    //1 means supervisor
    mstatus_r.set_bits(11..13, 1 as usize);
    mstatus::write(mstatus_r);
}
/// set MIE field
pub unsafe fn set_mie() {
    let mut mstatus_r = mstatus::read();
    mstatus_r.set_bit(3, true);
    mstatus::write(mstatus_r);
}

pub unsafe fn to_supervisor(){
    medeleg::write(0xffff);
    mideleg::write(0xffff);
    sie::intr_on();
}

pub unsafe fn allo_cpu(){
    let id = mhartid::read();
    tp::write(id);
}