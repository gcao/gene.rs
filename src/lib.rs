#![allow(unused_imports)]
#![allow(clippy::needless_return)]
#![allow(clippy::new_without_default)]
#![allow(clippy::new_without_default_derive)]

#![feature(range_contains)]
#![feature(box_patterns)]

pub mod compiler;
pub mod parser;
pub mod types;
pub mod utils;
pub mod vm;
