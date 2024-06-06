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

pub fn deserialize_input(buf: &[u8]) -> Result<interface::Input, prost::DecodeError> {
    interface::Input::decode(&mut std::io::Cursor::new(buf))
}

pub fn serialize_output(output: &interface::Output) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.reserve(output.encoded_len());
    // Unwrap is safe, since we have reserved sufficient capacity in the vector.
    output.encode(&mut buf).unwrap();
    buf
}
