#![allow(unused_imports)]
#![allow(clippy::needless_return)]
#![allow(clippy::new_without_default)]
#![allow(clippy::new_without_default)]

#![feature(box_patterns)]

#[macro_use]
pub mod utils;

pub mod parser;
pub mod types;
pub mod compiler;
pub mod compilable;
pub mod vm;
