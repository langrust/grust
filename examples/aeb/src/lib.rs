#![allow(warnings)]

// `aeb` module, where grust is used.
pub mod aeb;

// include the `interface` module, which is generated from interface.proto.
pub mod interface {
    tonic::include_proto!("interface");
}

// AEB `client` module.
pub mod client;

// AEB `server` module.
pub mod server;

// `json` IO module
pub mod json;

#[cfg(test)]
mod macro_output;
