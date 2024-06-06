#![allow(warnings)]


use prost::Message;

// include the `interface` module, which is generated from interface.proto.
pub mod interface {
    include!(concat!(env!("OUT_DIR"), "/interface.rs"));
}

// include the `aeb` module, where grust is used.
pub mod aeb;

#[cfg(test)]
mod macro_output;
use interface::*;
