use alloc::collections::BinaryHeap;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::{cmp, ops};

use crate::input::{Dir, Input, Turn};
use crate::matrix::Mat;
use crate::piece::{Cells, FallingPiece, Pos, Shape, Spawn, WallKicks};

type HashSet<T> = hashbrown::HashSet<T, core::hash::BuildHasherDefault<ahash::AHasher>>;

// == iterating all reachable places ==

/// Returns an iterator that yields all of the reachable places on `matrix` from piece
/// `piece_type`, starting at its spawn location. If the spawn location is blocked then
/// this will be empty (or you can check with `is_dead`).
pub fn places<T: Shape + Clone + Spawn>(matrix: &Mat, piece: T) -> Places<'_, T> {
    let mut places = Places {
        matrix,
        piece: piece.clone(),
        stack: Vec::with_capacity(64),
        visited: HashSet::with_capacity(256),
    };

    let spawn_fp = FallingPiece::spawn(piece);
    if !spawn_fp.cells().collides(matrix) {
        // if this is not true, then we are dead
        places.push(spawn_fp.pos);
    }

    places
}

#[derive(Clone)]
pub struct Places<'m, T: Shape + Clone> {
    matrix: &'m Mat,
    piece: T,
    stack: Vec<Pos>,
    visited: HashSet<Pos>,
}

impl<T: Shape + Clone> Places<'_, T> {
    /// Returns true if the player is dead since the piece spawn was blocked.
    pub fn is_dead(&self) -> bool {
        self.stack.is_empty() && self.visited.is_empty()
    }

    fn push(&mut self, pos: Pos) -> bool {
        if !self.visited.insert(pos) {
            return false;
        }
        self.stack.push(pos);
        true
    }

    fn pop(&mut self) -> Option<FallingPiece<T>> {
        Some(FallingPiece::new(self.piece.clone(), self.stack.pop()?))
    }
}

/// Value returned by the `Places` iterator, describing the final location and if this
/// location is immobile.
#[derive(Copy, Clone, Debug)]
pub struct PlacesResult<T> {
    pub falling_piece: FallingPiece<T>,
    pub cells: Cells,
}

impl<T> From<PlacesResult<T>> for FallingPiece<T> {
    fn from(r: PlacesResult<T>) -> Self {
        r.falling_piece
    }
}

impl<T> ops::Deref for PlacesResult<T> {
    type Target = FallingPiece<T>;
    fn deref(&self) -> &Self::Target {
        &self.falling_piece
    }
}

impl<T: Shape + Clone + WallKicks> Iterator for Places<'_, T> {
    type Item = PlacesResult<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut piece = self.pop()?;

            let mut cw = piece.clone();
            if cw.try_rotate(self.matrix, Turn::Cw).is_some() {
                self.push(cw.pos);
            }

            let mut ccw = piece.clone();
            if ccw.try_rotate(self.matrix, Turn::Ccw).is_some() {
                self.push(ccw.pos);
            }

            let mut left = piece.clone();
            while left.try_shift(self.matrix, Dir::Left).is_some() {
                if !self.push(left.pos) {
                    break;
                }
            }

            let mut right = piece.clone();
            while right.try_shift(self.matrix, Dir::Right).is_some() {
                if !self.push(right.pos) {
                    break;
                }
            }

            let (dy, cells) = piece.sonic_drop(self.matrix);
            if dy != 0 {
                // piece was floating so we don't return it yet; push it on the stack and
                // return it when reached in a future iteration
                self.push(piece.pos);
                continue;
            }

            return Some(PlacesResult {
                falling_piece: piece,
                cells,
            });
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::input::Rot;
    use crate::matrix::MatBuf;
    use crate::standard_rules;
    use crate::test::assert_same_set;
    use core::fmt;
    use core::ops::RangeInclusive;

    fn assert_places<T>(
        piece: T,
        mat: &Mat,
        expected: impl IntoIterator<Item = (RangeInclusive<i8>, i8, Rot)>,
        immobile: impl IntoIterator<Item = (i8, i8, Rot)>,
    ) where
        T: Shape + Spawn + WallKicks + Copy + fmt::Display,
    {
        let immobile = immobile.into_iter().map(Pos::from).collect::<Vec<_>>();
        let actual_places = places(mat, piece).map(|f| {
            let pos = f.pos;
            let imm = f.cells.immobile(mat);
            assert_eq!(imm, immobile.contains(&pos), "{pos:?}");
            pos
        });
        let expected_places = expected
            .into_iter()
            .flat_map(|(xs, y, r)| xs.map(move |x| (x, y, r).into()));
        assert_same_set(actual_places, expected_places, &piece);
    }

    #[test]
    fn test_t_places() {
        assert_places(
            standard_rules::T,
            Mat::empty(),
            [
                (0..=7, 1, Rot::N),
                (-1..=7, 2, Rot::E),
                (0..=7, 2, Rot::S),
                (0..=8, 2, Rot::W),
            ],
            [],
        );
    }

    #[test]
    fn test_i_places() {
        assert_places(
            standard_rules::I,
            Mat::empty(),
            [
                (0..=6, 1, Rot::N),
                (0..=6, 2, Rot::S), // same as N
                (-2..=7, 3, Rot::E),
                (-1..=8, 3, Rot::W), // same as W
            ],
            [],
        );
    }

    #[test]
    fn test_s_spin_places() {
        let mut mat = MatBuf::new();
        // 1 xxxxx..xxx
        // 0 xxxx..xxxx
        //   0123456789
        mat.set(0, 0b1111001111);
        mat.set(1, 0b1110011111);
        //
        assert_places(
            standard_rules::S,
            &mat,
            [
                (0..=4, 3, Rot::N),
                (5..=5, 2, Rot::N),
                (6..=7, 3, Rot::N),
                (0..=4, 4, Rot::S),
                (5..=5, 3, Rot::S),
                (6..=7, 4, Rot::S),
                (-1..=2, 4, Rot::E),
                (3..=4, 3, Rot::E),
                (5..=7, 4, Rot::E),
                (0..=3, 4, Rot::W),
                (4..=5, 3, Rot::W),
                (6..=8, 4, Rot::W),
                // spin
                (4..=4, 2, Rot::S),
            ],
            [(4, 2, Rot::S)],
        );
    }
}

// == finding shortest input sequences ==

/// Returns the minimal input sequence to reach `target` from the piece spawn location. If
/// a reachable path is not found then returns `None`.
///
/// The input sequence does *not* include the `SonicDrop` that would place it on the
/// ground; it is implied that this would be accomplished by a hard drop always performed.
pub fn reach<T>(matrix: &Mat, target: FallingPiece<T>) -> Option<Vec<Input>>
where
    T: Shape + Spawn + WallKicks + Clone,
{
    let target_cells = target.cells();
    ShortestPath::new(matrix, target.piece)
        .find(|(cells, _)| *cells == target_cells)
        .map(|(_, node)| node.inputs())
}

/// Implements Djikstra's Algorithm in order to list all shortest paths to reachable
/// places on a matrix.
struct ShortestPath<'m, T: Shape + Clone> {
    matrix: &'m Mat,
    piece_type: T,
    unvisited: BinaryHeap<ShortestPathNode>,
    visited: HashSet<Pos>,
}

impl<'m, T: Shape + Spawn + Clone> ShortestPath<'m, T> {
    fn new(matrix: &'m Mat, piece_type: T) -> Self {
        let spawn_piece = FallingPiece::spawn(piece_type.clone());
        let root = ShortestPathNode::new_root(spawn_piece.pos);
        Self {
            matrix,
            piece_type,
            unvisited: BinaryHeap::from_iter([root]),
            visited: HashSet::with_capacity(256),
        }
    }
}

impl<T: Shape + Clone> ShortestPath<'_, T> {
    fn push(&mut self, parent: &ShortestPathNode, input: Input, pos: Pos) {
        if self.visited.insert(pos) {
            self.unvisited.push(parent.new_child(input, pos));
        }
    }
}

impl<T: Shape + WallKicks + Clone> Iterator for ShortestPath<'_, T> {
    type Item = (Cells, ShortestPathNode);

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.unvisited.pop()?;
        let piece = FallingPiece::new(self.piece_type.clone(), node.pos);

        let mut cw = piece.clone();
        if cw.try_rotate(self.matrix, Turn::Cw).is_some() {
            self.push(&node, Input::Cw, cw.pos);
        }

        let mut ccw = piece.clone();
        if ccw.try_rotate(self.matrix, Turn::Ccw).is_some() {
            self.push(&node, Input::Ccw, ccw.pos);
        }

        let mut left = piece.clone();
        if left.try_shift(self.matrix, Dir::Left).is_some() {
            self.push(&node, Input::Left, left.pos);
        }

        let mut right = piece.clone();
        if right.try_shift(self.matrix, Dir::Right).is_some() {
            self.push(&node, Input::Right, right.pos);
        }

        let mut sd = piece;
        let (dy, cells) = sd.sonic_drop(self.matrix);
        if dy != 0 {
            self.push(&node, Input::SonicDrop, sd.pos);
        }

        Some((cells, node))
    }
}

#[derive(Clone)]
struct ShortestPathNode(Rc<ShortestPathNodeData>);

impl ops::Deref for ShortestPathNode {
    type Target = ShortestPathNodeData;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct ShortestPathNodeData {
    n_shift: u32,
    n_rotate: u32,
    n_drop: u32,
    pos: Pos,
    input: Option<Input>,
    parent: Option<ShortestPathNode>,
}

impl ShortestPathNode {
    fn new_root(pos: Pos) -> Self {
        Self(Rc::new(ShortestPathNodeData {
            n_shift: 0,
            n_rotate: 0,
            n_drop: 0,
            pos,
            input: None,
            parent: None,
        }))
    }

    fn new_child(&self, input: Input, pos: Pos) -> Self {
        let mut n_shift = self.n_shift;
        let mut n_rotate = self.n_rotate;
        let mut n_drop = self.n_drop;
        match input {
            Input::Left | Input::Right => n_shift += 1,
            Input::Cw | Input::Ccw => n_rotate += 1,
            Input::SonicDrop => n_drop += 1,
        }
        Self(Rc::new(ShortestPathNodeData {
            n_shift,
            n_rotate,
            n_drop,
            pos,
            input: Some(input),
            parent: Some(self.clone()),
        }))
    }
}

// this triple is used to compare length of input sequence, in order to break input-count
// ties by first minimizing number of drops, and then minimizing number of rotates. this
// makes it so that left/right inputs come first, then rotations, then drops.
type Distance = (u32, u32, u32);

impl ShortestPathNodeData {
    fn n_inputs(&self) -> u32 {
        self.n_shift + self.n_rotate + self.n_drop
    }

    fn distance(&self) -> Distance {
        (self.n_inputs(), self.n_drop, self.n_rotate)
    }

    fn inputs(&self) -> Vec<Input> {
        let mut inputs = Vec::with_capacity(self.n_inputs() as usize);
        let mut parent: Option<&ShortestPathNodeData> = Some(self);
        while let Some(node) = parent.take() {
            inputs.extend(node.input);
            parent = node.parent.as_deref();
        }
        inputs.reverse();
        inputs
    }
}

impl cmp::Ord for ShortestPathNode {
    fn cmp(&self, rhs: &Self) -> cmp::Ordering {
        rhs.distance().cmp(&self.distance())
    }
}

impl cmp::Eq for ShortestPathNode {}

impl cmp::PartialEq for ShortestPathNode {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl cmp::PartialOrd for ShortestPathNode {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod test_reach {
    use super::*;
    use crate::input::Rot;
    use crate::matrix::MatBuf;
    use crate::standard_rules;

    #[test]
    fn test_reach_simple() {
        let mat = Mat::empty();
        let tgt = FallingPiece::new(standard_rules::T, (0, 1, Rot::N));
        let inputs = reach(mat, tgt).unwrap();
        assert_eq!(inputs, {
            use Input::*;
            [Left, Left, Left]
        });
    }

    #[test]
    fn test_reach_simple_rotate() {
        let mat = Mat::empty();
        let tgt = FallingPiece::new(standard_rules::T, (-1, 2, Rot::E));
        let inputs = reach(mat, tgt).unwrap();
        assert_eq!(inputs, {
            use Input::*;
            [Left, Left, Left, Cw, Left]
        });
    }

    #[test]
    fn test_reach_s_spin() {
        let mut mat = MatBuf::new();
        // 1 xxxxx..xxx
        // 0 xxxx..xxxx
        //   0123456789
        mat.set(0, 0b1111001111);
        mat.set(1, 0b1110011111);
        let tgt = FallingPiece::new(standard_rules::S, (4, 2, Rot::S));
        let inputs = reach(&mat, tgt).unwrap();
        assert_eq!(inputs, {
            use Input::*;
            [Cw, SonicDrop, Cw]
        });
    }
}
