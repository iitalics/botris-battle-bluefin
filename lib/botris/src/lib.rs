//! Interface to Botris Battle API.

#![allow(clippy::new_without_default)]

#[macro_use]
extern crate tracing;

pub mod api;
pub use api::*;

pub mod game;
pub use game::*;
