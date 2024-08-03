//! Tetris implementation for Botris.

use serde::{Deserialize, Serialize};
use std::num::NonZeroU8;

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct Board(pub Vec<[Block; 10]>);

impl Board {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AsRef<[[Block; 10]]> for Board {
    fn as_ref(&self) -> &[[Block; 10]] {
        &self.0
    }
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn row_to_string(bs: &[Block]) -> String {
            bs.iter().map(|b| b.map_or("_", |b| b.name())).collect()
        }

        f.debug_list()
            .entries(self.0.iter().map(|bs| row_to_string(bs)))
            .finish()
    }
}

pub type Queue = Vec<Piece>;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PieceData {
    pub piece: Piece,
    pub rotation: Rotation,
    pub x: i8,
    pub y: i8,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[repr(u8)]
pub enum Piece {
    I = 1,
    O = 2,
    J = 3,
    L = 4,
    S = 5,
    Z = 6,
    T = 7,
}

impl Piece {
    pub fn name(self) -> &'static str {
        BLOCK_NAMES[self as usize]
    }
}

impl From<Piece> for u8 {
    fn from(v: Piece) -> Self {
        v as u8
    }
}

impl From<Piece> for NonZeroU8 {
    fn from(v: Piece) -> Self {
        NonZeroU8::new(v.into()).unwrap()
    }
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[repr(u8)]
pub enum NonEmptyBlock {
    I = 1,
    O = 2,
    J = 3,
    L = 4,
    S = 5,
    Z = 6,
    T = 7,
    G = 8,
}

impl NonEmptyBlock {
    pub fn name(self) -> &'static str {
        BLOCK_NAMES[self as usize]
    }
}

impl Default for NonEmptyBlock {
    fn default() -> Self {
        NonEmptyBlock::G
    }
}

impl From<Piece> for NonEmptyBlock {
    fn from(v: Piece) -> Self {
        unsafe { std::mem::transmute(v) }
    }
}

impl From<NonEmptyBlock> for u8 {
    fn from(v: NonEmptyBlock) -> Self {
        v as u8
    }
}

impl From<NonEmptyBlock> for NonZeroU8 {
    fn from(v: NonEmptyBlock) -> Self {
        NonZeroU8::new(v.into()).unwrap()
    }
}

impl std::fmt::Display for NonEmptyBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

pub type Block = Option<NonEmptyBlock>;

static BLOCK_NAMES: [&str; 9] = ["", "I", "O", "J", "L", "S", "Z", "T", "G"];

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
#[repr(u8)]
pub enum Rotation {
    #[default]
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl From<u8> for Rotation {
    fn from(v: u8) -> Self {
        unsafe { std::mem::transmute(v & 3) }
    }
}

impl From<Rotation> for u8 {
    fn from(r: Rotation) -> Self {
        r as u8
    }
}

impl Serialize for Rotation {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        u8::from(*self).serialize(ser)
    }
}
impl<'de> Deserialize<'de> for Rotation {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        u8::deserialize(de).map(Rotation::from)
    }
}

impl std::fmt::Display for Rotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}
