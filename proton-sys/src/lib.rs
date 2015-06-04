#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate libc;
#[macro_use] extern crate bitflags;

type __builtin_va_list = libc::c_void;

pub const PN_EOS: i8 = -1;
pub const PN_ERR: i8 = -2;
pub const PN_OVERFLOW: i8 = -3;
pub const PN_UNDERFLOW: i8 = -4;
pub const PN_STATE_ERR: i8 = -5;
pub const PN_ARG_ERR: i8 = -6;
pub const PN_TIMEOUT: i8 = -7;
pub const PN_INTR: i8 = -8;
pub const PN_INPROGRESS: i8 = -9;


bitflags! {
    flags StateFlags: i32 {
        const LOCAL_UNINIT         = 0b00000001,
        const LOCAL_ACTIVE         = 0b00000010,
        const LOCAL_CLOSED         = 0b00000100,
        const REMOTE_UNINIT        = 0b00001000,
        const REMOTE_ACTIVE        = 0b00010000,
        const REMOTE_CLOSED        = 0b00100000,
        const LOCAL_MASK           = LOCAL_UNINIT.bits
                                   | LOCAL_ACTIVE.bits
                                   | LOCAL_CLOSED.bits,
        const REMOTE_MASK          = REMOTE_UNINIT.bits
                                   | REMOTE_ACTIVE.bits
                                   | REMOTE_CLOSED.bits,
    }
}

impl StateFlags {
    pub fn local_state(&self) -> StateFlags {
        *self & LOCAL_MASK
    }

    pub fn remote_state(&self) -> StateFlags {
        *self & REMOTE_MASK
    }
}

include!("ffi.rs");
