// TODO: remove these when everything is implemented
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

//extern crate mio;
extern crate libc;
extern crate rustc_serialize;
extern crate proton_sys;

#[macro_use]
extern crate log;

pub use proton::{
    Transport
};

//mod io;
mod proton;
