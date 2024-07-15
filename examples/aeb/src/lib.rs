#![allow(warnings)]

use grust::grust;
pub mod macro_output;
pub mod output;

grust! {
    #![dump = "examples/aeb/src/macro_output.rs", greusot = true]

#[cfg(test)]
mod macro_output;

#[cfg(test)]
mod output;
