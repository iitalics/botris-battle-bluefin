#[macro_use]
extern crate tracing;

use bumpalo::Bump as Alo;
use std::cell::Cell;
use std::{cmp, mem, time};

use mino::input::Input;
use mino::matrix::{Mat, MatBuf};
use mino::places::{places, reach};
use mino::standard_rules::{FallingPiece, Piece, Queue};

const K: usize = 1024;
const M: usize = 1024 * K;

pub fn bot(
    current: Piece,
    queue: &[Piece],
    hold: Option<Piece>,
    matrix: &Mat,
    // b2b: bool,
    // ren: u32,
    // jeopardy: [u32; 2],
) -> Option<(bool, Vec<Input>)> {
    let t0 = time::Instant::now();
    let max_time = time::Duration::from_millis(150);

    let combined_queue_pieces: Vec<Piece> = [hold.as_slice(), &[current], queue]
        .into_iter()
        .flat_map(|x| x.iter().copied())
        .collect();
    let queue = Queue::from(&*combined_queue_pieces);

    debug!("start {:?}", queue);

    let alo = Alo::with_capacity(16 * M);

    let root = Node::alloc_root(&alo, matrix, queue);
    let mut best = root;

    let mut beam_width = 16;
    let mut beam = Vec::with_capacity(8 * K);
    let mut next_beam = Vec::with_capacity(8 * K);

    let mut total_expanded = 1;
    let mut best_expanded = 1;

    while t0.elapsed() < max_time {
        trace!(beam_width, best = best.score);
        beam.clear();
        beam.push(root);

        let mut depth = 0;
        let mut expanded = 0;
        let mut not_expanded = 0;

        while beam.len() > 0 {
            if beam.len() > beam_width {
                beam.select_nth_unstable_by_key(beam_width, |node| cmp::Reverse(node.score));
                beam.truncate(beam_width);
            }

            for &node in beam.iter() {
                if node.score > best.score {
                    trace!(new_best = node.score);
                    best = node;
                    best_expanded = total_expanded;
                }

                if node.children.get().is_empty() {
                    let children = node.alloc_chilren(&alo);
                    node.children.set(alo.alloc_slice_copy(&children));
                    expanded += node.children.get().len();
                    total_expanded += expanded;
                } else {
                    not_expanded += node.children.get().len();
                }

                next_beam.extend_from_slice(node.children.get());
            }

            beam.clear();
            mem::swap(&mut beam, &mut next_beam);

            trace!(depth, expanded, not_expanded);
            depth += 1;
            expanded = 0;
            not_expanded = 0;
        }

        beam_width *= 2;
    }

    info!(best = best.score, total_expanded, best_expanded, beam_width);

    let mut target = None;
    while let Some((parent, edge)) = best.parent {
        target = Some(edge);
        best = parent;
    }

    debug!(alo_kb = alo.allocated_bytes() / 1024);

    let target = target?;
    debug!("  -> {:?}", target);

    let hold = target.piece != current;
    let reach_inputs = reach(matrix, target)?;
    Some((hold, reach_inputs))
}

struct Node<'a> {
    matrix: &'a Mat,
    queue: Queue<'a>,
    depth: i32,
    score: i32,
    parent: Option<(&'a Node<'a>, FallingPiece)>,
    children: Cell<&'a [&'a Node<'a>]>,
}

fn copy_matrix<'a>(alo: &'a Alo, mat: &Mat) -> &'a Mat {
    Mat::new(alo.alloc_slice_copy(mat.rows()))
}

impl<'a> Node<'a> {
    fn alloc_root(alo: &'a Alo, matrix: &Mat, queue: Queue<'a>) -> &'a Node<'a> {
        let matrix = copy_matrix(alo, matrix);
        // let queue = copy_queue(alo, queue);
        alo.alloc_with(|| Node {
            matrix,
            queue,
            depth: 0,
            score: i32::MIN,
            parent: None,
            children: Cell::new(&[]),
        })
    }

    fn alloc_chilren(&'a self, alo: &'a Alo) -> Vec<&'a Node<'a>> {
        let mut children = vec![];
        let mut new_matrix = MatBuf::new();

        for (pc, queue) in self.queue.pop() {
            for fp in places(self.matrix, pc) {
                // let is_spin = pc.cells.immobile(self.matrix);
                new_matrix.copy_from(self.matrix);
                fp.cells.place(&mut new_matrix);
                let cleared = new_matrix.clear_lines(fp.cells.bottom());

                // TODO: dedup
                let matrix = copy_matrix(alo, &new_matrix);
                let depth = self.depth + 1;
                let parent = Some((self, fp.into()));
                let score = evaluate(matrix, depth, cleared);
                children.push(alo.alloc_with(|| Node {
                    matrix,
                    queue,
                    depth,
                    score,
                    parent,
                    children: Cell::new(&[]),
                }) as &Node);
            }
        }

        children
    }
}

fn evaluate(mat: &Mat, placed: i32, cleared: u8) -> i32 {
    let height = mat.len() as i32;
    let row_trans = row_transitions(mat);
    let blocks = count_blocks(mat);
    let blocks_from_target = (blocks - 36).abs();

    let mut score = 16384;
    score -= 100 * placed;
    score -= 100 * height;
    score -= 40 * blocks_from_target;
    score -= 600 * row_trans;
    if cleared == 1 {
        score -= 200;
    }
    if cleared == 3 {
        score += 100;
    }
    if cleared == 4 {
        score += 1000;
    }

    score
}

fn count_blocks(mat: &Mat) -> i32 {
    mat.rows().iter().map(|&r| r.count_ones() as i32).sum()
}

fn row_transitions(mat: &Mat) -> i32 {
    let mut trans = 0;
    let mut prev = mino::matrix::FULL;
    for &row in mat.rows() {
        trans += (row ^ prev).count_ones() as i32;
        prev = row;
    }
    // trans += (mino::matrix::EMPTY ^ prev).count_ones() as i32;
    trans += (prev << 6).count_ones() as i32;
    trans
}
