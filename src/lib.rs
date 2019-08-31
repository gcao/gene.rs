#![allow(unused_imports)]
#![allow(clippy::needless_return)]
#![allow(clippy::new_without_default)]
#![allow(clippy::new_without_default)]

#![feature(box_patterns)]

#[macro_use]
pub mod utils;

pub mod benchmarker;
pub mod compiler;
pub mod parser;
pub mod types;
pub mod compiler2;
pub mod vm;
