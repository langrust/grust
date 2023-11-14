pub enum Color {
    Blue,
    Red,
    Yellow,
    Green,
    Purple,
}
pub fn foo(x: i64, y: i64) -> i64 {
    let mut z = x + y;
    z = z + 1i64;
    z
}
impl Display for Point {
    type MyString = String;
    fn fmt(&self, f: &mut String) {
        write!(f, "({}, {})", self.x, self.y);
    }
}
pub use std::fmt::Debug;
pub use std::future::Future as AliasFuture;
pub use std::sync::*;
pub use std::{sync::*, fmt::Debug, future::Future as AliasFuture};
pub mod my_module;
pub struct Point {
    pub x: i64,
    pub y: i64,
    z: i64,
}
pub trait Display {
    type MyString;
    fn fmt(&self, f: &mut String);
}
pub type Integer = i64;
