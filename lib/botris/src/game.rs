//! Tetris implementation for Botris.

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct Board(pub Vec<[Block; 10]>);

impl Board {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rows(&self) -> &[[Block; 10]] {
        &self.0
    }

    pub fn len(&self) -> i8 {
        self.rows().len() as i8
    }

    pub fn is_empty(&self) -> bool {
        self.rows().is_empty()
    }

    pub fn check_collision(&self, piece_data: PieceData) -> bool {
        piece_data.coords().any(|xy| self[xy].is_some())
    }

    pub fn check_immobile(&self, piece_data: PieceData) -> bool {
        for ofs in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
            if self.check_collision(piece_data.offset(ofs)) {
                return true;
            }
        }
        false
    }

    pub fn place_piece(&mut self, piece_data: PieceData) {
        let block = Some(piece_data.piece.into());
        piece_data.coords().for_each(|xy| self[xy] = block);
    }

    pub fn clear_lines(&mut self) -> i8 {
        let mut cleared = 0;
        let rows = &mut self.0;
        let mut y_dst = 0;
        for y in 0..rows.len() {
            let is_full = !rows[y].contains(&None);
            //let is_garbage = rows[y].contains(&Some(NonEmptyBlock::G));
            if is_full {
                cleared += 1;
            } else {
                rows.swap(y_dst, y);
                y_dst += 1;
            }
        }
        rows.truncate(y_dst);
        cleared
    }
}

impl std::ops::Index<(i8, i8)> for Board {
    type Output = Block;
    fn index(&self, (x, y): (i8, i8)) -> &Block {
        if !(0..10).contains(&x) || y < 0 {
            return &Some(NonEmptyBlock::G);
        }
        if y >= self.len() {
            return &None;
        }
        &self.rows()[y as usize][x as usize]
    }
}

impl std::ops::IndexMut<(i8, i8)> for Board {
    fn index_mut(&mut self, (x, y): (i8, i8)) -> &mut Block {
        if !(0..10).contains(&x) || y < 0 {
            panic!("board index out of bounds");
        }
        let x = x as usize;
        let y = y as usize;
        let rows = &mut self.0;
        while y >= rows.len() {
            rows.push([None; 10]);
        }
        &mut rows[y][x]
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

pub type Queue = VecDeque<Piece>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PieceData {
    pub piece: Piece,
    pub rotation: Rotation,
    pub x: i8,
    pub y: i8,
}

impl PieceData {
    pub fn spawn(piece: Piece) -> Self {
        Self {
            piece,
            rotation: Rotation::North,
            x: 5 - (piece.width() + 1) / 2,
            y: 20,
        }
    }

    pub fn offset(self, (dx, dy): (i8, i8)) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            ..self
        }
    }

    pub fn rotate(self, r: Rotation) -> Self {
        Self {
            rotation: r,
            ..self
        }
    }

    pub fn try_offset(&mut self, ofs: (i8, i8), board: &Board) -> bool {
        let moved = self.offset(ofs);
        if !board.check_collision(moved) {
            *self = moved;
            return true;
        }
        false
    }

    pub fn try_rotate_cw(&mut self, board: &Board) -> bool {
        self.try_rotate(self.rotation.cw(), board)
    }

    pub fn try_rotate_ccw(&mut self, board: &Board) -> bool {
        self.try_rotate(self.rotation.ccw(), board)
    }

    fn try_rotate(&mut self, new_r: Rotation, board: &Board) -> bool {
        let old_r = self.rotation;
        for ofs in self.piece.wall_kicks(old_r, new_r) {
            let kicked = self.rotate(new_r).offset(ofs);
            if !board.check_collision(kicked) {
                *self = kicked;
                return true;
            }
        }
        false
    }

    pub fn sonic_drop(&mut self, board: &Board) -> i8 {
        let mut dy = 0;
        loop {
            let drop = self.offset((0, dy - 1));
            if board.check_collision(drop) {
                break;
            }
            dy -= 1;
        }
        *self = self.offset((0, dy));
        dy
    }

    pub fn coords(self) -> impl Iterator<Item = (i8, i8)> {
        self.piece.north_coords().map(move |xy| {
            let (dx, dy) = rotate(self.rotation, self.piece.width(), xy);
            (self.x + dx, self.y - dy)
        })
    }
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

pub static ALL_PIECES: [Piece; 7] = {
    use Piece::*;
    [I, O, J, L, S, Z, T]
};

impl Piece {
    pub fn name(self) -> &'static str {
        BLOCK_NAMES[self as usize]
    }

    fn width(self) -> i8 {
        match self {
            Piece::I => 4,
            Piece::O => 2,
            _ => 3,
        }
    }

    fn north_coords(self) -> impl Iterator<Item = (i8, i8)> {
        match self {
            Piece::I => &[(0, 1), (1, 1), (2, 1), (3, 1)],
            Piece::O => &[(0, 0), (0, 1), (1, 0), (1, 1)],
            Piece::J => &[(0, 0), (0, 1), (1, 1), (2, 1)],
            Piece::L => &[(0, 1), (1, 1), (2, 0), (2, 1)],
            Piece::S => &[(0, 1), (1, 0), (1, 1), (2, 0)],
            Piece::Z => &[(0, 0), (1, 0), (1, 1), (2, 1)],
            Piece::T => &[(0, 1), (1, 0), (1, 1), (2, 1)],
        }
        .iter()
        .copied()
    }

    fn wall_kicks(self, r0: Rotation, r1: Rotation) -> impl Iterator<Item = (i8, i8)> {
        if self == Piece::I {
            match (r0 as u8, r1 as u8) {
                (0, 1) => &[(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
                (1, 0) => &[(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
                (1, 2) => &[(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
                (2, 1) => &[(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
                (2, 3) => &[(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
                (3, 2) => &[(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
                (3, 0) => &[(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
                (_, _) => &[(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
            }
        } else {
            match (r0 as u8, r1 as u8) {
                (0, 1) => &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                (1, 0) => &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                (1, 2) => &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                (2, 1) => &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                (2, 3) => &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
                (3, 2) => &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
                (3, 0) => &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
                (_, _) => &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
            }
        }
        .iter()
        .copied()
    }
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[repr(u8)]
pub enum NonEmptyBlock {
    I = 1,
    O = 2,
    J = 3,
    L = 4,
    S = 5,
    Z = 6,
    T = 7,
    #[default]
    G = 8,
}

impl NonEmptyBlock {
    pub fn name(self) -> &'static str {
        BLOCK_NAMES[self as usize]
    }
}

impl From<Piece> for NonEmptyBlock {
    fn from(v: Piece) -> Self {
        unsafe { std::mem::transmute(v) }
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

impl Rotation {
    pub fn cw(self) -> Self {
        (self as u8 + 1).into()
    }

    pub fn ccw(self) -> Self {
        (self as u8 + 3).into()
    }
}

fn rotate(r: Rotation, w: i8, mut xy: (i8, i8)) -> (i8, i8) {
    for _ in 0..(r as u8) {
        // turn cw
        let (x, y) = xy;
        xy = (w - y - 1, x);
    }
    xy
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub board: Board,
    pub queue: Queue,
    pub garbage_queued: VecDeque<GarbageLine>,
    pub held: Option<Piece>,
    pub current: PieceData,
    pub can_hold: bool,
    pub combo: u32,
    pub b2b: bool,
    pub score: u32,
    pub pieces_placed: u32,
    pub dead: bool,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct GarbageLine {
    pub delay: u32,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Command {
    Hold,
    MoveLeft,
    MoveRight,
    RotateCw,
    RotateCcw,
    Drop,
    SonicDrop,
    HardDrop,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub state: GameState,
    rng: SmallRng,
    bag: Vec<Piece>,
}

impl std::ops::Deref for Game {
    type Target = GameState;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl Game {
    pub fn new() -> Self {
        Self::new_seeded(rand::thread_rng().gen())
    }

    pub fn new_seeded(s: u64) -> Self {
        Self::with_rng(SmallRng::seed_from_u64(s))
    }

    fn with_rng(rng: SmallRng) -> Self {
        let mut this = Game {
            state: GameState {
                board: Board::new(),
                queue: Queue::with_capacity(6),
                garbage_queued: VecDeque::new(),
                held: None,
                current: PieceData::spawn(Piece::O),
                can_hold: true,
                combo: 0,
                b2b: false,
                score: 0,
                pieces_placed: 0,
                dead: false,
            },
            rng,
            bag: Vec::with_capacity(7),
        };

        this.spawn_piece();
        this
    }

    // TODO: return list of attacks
    pub fn perform_commands(&mut self, cmds: &[Command]) {
        for &cmd in cmds.iter().chain([&Command::HardDrop]) {
            if !self.perform_command(cmd) {
                warn!("command blocked: {cmd:?} ({:?})", self.current);
            }
        }
    }

    pub fn perform_command(&mut self, cmd: Command) -> bool {
        match cmd {
            Command::MoveLeft => self.state.current.try_offset((-1, 0), &self.state.board),
            Command::MoveRight => self.state.current.try_offset((1, 0), &self.state.board),
            Command::Drop => self.state.current.try_offset((-1, 0), &self.state.board),
            Command::RotateCw => self.state.current.try_rotate_cw(&self.state.board),
            Command::RotateCcw => self.state.current.try_rotate_ccw(&self.state.board),
            Command::SonicDrop => self.state.current.sonic_drop(&self.state.board) > 0,

            Command::Hold => {
                if !self.can_hold {
                    return false;
                }

                if let Some(held) = self.state.held {
                    self.state.queue.push_front(held);
                } else if self.state.queue.is_empty() {
                    return false;
                }

                self.state.held = Some(self.state.current.piece);
                self.spawn_piece();
                self.state.can_hold = false;
                true
            }

            Command::HardDrop => {
                self.state.current.sonic_drop(&self.state.board);
                let immobile = self.state.board.check_immobile(self.state.current);
                self.state.board.place_piece(self.current);
                let cleared = self.state.board.clear_lines();

                let score = calculate_score(
                    cleared,
                    immobile,
                    &mut self.state.b2b,
                    &mut self.state.combo,
                );

                /* TODO: apply multiplier to score */

                let (_cancel, _attack) = self.cancel_garbage(score);

                self.state.score += score;
                self.state.pieces_placed += 1;
                true
            }
        }
    }

    fn spawn_piece(&mut self) {
        while self.queue.len() < 7 {
            if self.bag.is_empty() {
                self.bag.extend_from_slice(&ALL_PIECES);
            }
            let i = self.rng.gen_range(0..self.bag.len());
            let pc = self.bag.swap_remove(i);
            self.state.queue.push_back(pc);
        }

        let piece = self.state.queue.pop_front().unwrap();
        self.state.current = PieceData::spawn(piece);
        self.state.dead = self.state.board.check_collision(self.state.current);
        self.state.can_hold = true;
    }

    fn cancel_garbage(&mut self, attack: u32) -> (u32, u32) {
        let cancel = self.garbage_queued.len().min(attack as usize);
        self.state.garbage_queued.drain(..cancel);
        (cancel as u32, attack - cancel as u32)
    }
}

pub mod score {
    pub const SINGLE: u32 = 0;
    pub const DOUBLE: u32 = 1;
    pub const TRIPLE: u32 = 2;
    pub const QUAD: u32 = 4;
    pub const SPIN_SINGLE: u32 = 2;
    pub const SPIN_DOUBLE: u32 = 4;
    pub const SPIN_TRIPLE: u32 = 6;
    pub const B2B: u32 = 1;
    pub const MAX_COMBO: usize = 9;
    pub const COMBO: [u32; 1 + MAX_COMBO] = [0, 0, 1, 1, 1, 2, 2, 3, 3, 4];
}

fn calculate_score(cleared: i8, is_immobile: bool, b2b: &mut bool, combo: &mut u32) -> u32 {
    if cleared == 0 {
        *combo = 0;
        return 0;
    }

    let mut score;
    let b2b_clear;

    if is_immobile {
        match cleared {
            1 => score = score::SPIN_SINGLE,
            2 => score = score::SPIN_DOUBLE,
            /* 3, 4 */ _ => score = score::SPIN_TRIPLE,
        }
        b2b_clear = true;
    } else {
        match cleared {
            1 => score = score::SINGLE,
            2 => score = score::DOUBLE,
            3 => score = score::TRIPLE,
            /* 4 */ _ => score = score::QUAD,
        }
        b2b_clear = cleared >= 4;
    }

    if b2b_clear && *b2b {
        score += score::B2B;
    }
    *b2b = b2b_clear;

    score += score::COMBO[*combo as usize];
    *combo = (*combo + 1).min(score::MAX_COMBO as u32);

    score
}
