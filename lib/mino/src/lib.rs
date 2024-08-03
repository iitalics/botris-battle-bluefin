#![no_std]

extern crate alloc;

#[cfg(test)]
extern crate std;

pub mod matrix;
pub use matrix::{Mat, MatBuf};

pub mod piece;
pub use piece::{Cells, Pos, Rot};

pub mod standard_rules;
