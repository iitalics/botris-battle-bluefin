use crate::Alloc;

use mino::matrix::{Mat, MatBuf};
use mino::places::places;
use mino::standard_rules::{FallingPiece, Queue};
use std::cell::Cell;

use crate::eval::evaluate;
use crate::state::State;

pub struct Node<'a> {
    matrix: &'a Mat,
    queue: Queue<'a>,
    score: i32,
    state: State,
    // depth: i32,
    // score: i32,
    parent: Option<Edge<'a>>,
    children: Cell<&'a [&'a Node<'a>]>,
}

#[derive(Copy, Clone)]
struct Edge<'a> {
    node: &'a Node<'a>,
    piece: FallingPiece,
    // cleared: u8,
    // is_spin: bool,
}

fn copy_matrix<'a>(alo: &'a Alloc, mat: &Mat) -> &'a Mat {
    Mat::new(alo.alloc_slice_copy(mat.rows()))
}

impl<'a> Node<'a> {
    pub fn root(alo: &'a Alloc, matrix: &'a Mat, queue: Queue<'a>, b2b: bool) -> &'a Self {
        alo.alloc_with(|| Node {
            matrix,
            queue,
            state: State::new(b2b),
            score: i32::MIN,
            parent: None,
            children: Cell::new(&[]),
        })
    }

    pub fn score(&self) -> i32 {
        self.score
    }

    pub fn state(&self) -> State {
        self.state
    }

    // if `n.ord_max_best() < m.ord_max_best()` then `n` is better than `m`
    pub fn ord_min_best(&self) -> std::cmp::Reverse<i32> {
        std::cmp::Reverse(self.ord_max_best())
    }

    // if `n.ord_max_best() > m.ord_max_best()` then `n` is better than `m`
    pub fn ord_max_best(&self) -> i32 {
        self.score()
    }

    pub fn original_piece(&self) -> Option<FallingPiece> {
        let mut edge = self.parent;
        let mut target = None;
        while let Some(e) = edge {
            target = Some(e.piece);
            edge = e.node.parent;
        }
        target
    }

    pub fn children(&self) -> &'a [&'a Node<'a>] {
        self.children.get()
    }

    pub fn expand(&'a self, alo: &'a Alloc) -> &'a [&'a Node<'a>] {
        let mut children = Vec::with_capacity(64);
        let mut new_matrix = MatBuf::new();

        for (pc, queue) in self.queue.pop() {
            for pl in places(self.matrix, pc) {
                new_matrix.copy_from(self.matrix);
                let is_spin = pl.cells.immobile(&new_matrix);
                new_matrix.place(pl.cells);
                let cleared = new_matrix.clear_lines(pl.cells.bottom());
                let state = self.state.next(cleared, is_spin);

                // TODO: check transposition table

                let matrix = copy_matrix(alo, &new_matrix);
                let score = evaluate(matrix, state);

                children.push(alo.alloc_with(move || Node {
                    matrix,
                    queue,
                    score,
                    state,
                    parent: Some(Edge {
                        node: self,
                        piece: pl.falling_piece,
                        // cleared,
                        // is_spin,
                    }),
                    children: Cell::new(&[]),
                }) as &Node);
            }
        }

        let children = alo.alloc_slice_copy(&children);
        self.children.set(children);
        children
    }
}
