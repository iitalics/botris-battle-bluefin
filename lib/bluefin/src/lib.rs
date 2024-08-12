#[macro_use]
extern crate tracing;

pub use bumpalo::Bump as Alloc;

use std::mem::swap;

use mino::input::Input;
use mino::matrix::Mat;
use mino::places::reach;
use mino::standard_rules::{Piece, Queue};

mod dag;
mod eval;
mod state;

const INITIAL_HEAP_CAPACITY: usize = 32 * 1024 * 1024;
const INITIAL_BEAM_CAPACITY: usize = 32 * 1024 * 1024;

pub fn bot(
    current: Piece,
    queue: &[Piece],
    hold: Option<Piece>,
    matrix: &Mat,
    // b2b: bool,
    // ren: u32,
    // jeopardy: [u32; 2],
) -> Option<(bool, Vec<Input>)> {
    let combined_queue_pieces: Vec<Piece> = [hold.as_slice(), &[current], queue]
        .into_iter()
        .flat_map(|x| x.iter().copied())
        .collect();
    let queue = Queue::from(&*combined_queue_pieces);

    debug!("start {}", queue);

    let alo = Alloc::with_capacity(INITIAL_HEAP_CAPACITY);

    let mut beam = Vec::with_capacity(INITIAL_BEAM_CAPACITY);
    let mut next_beam = Vec::with_capacity(INITIAL_BEAM_CAPACITY);

    let b2b = false; // TODO
    let root = dag::Node::root(&alo, matrix, queue, b2b);

    let mut total_expanded = 1;

    let mut best = root;
    //let mut best_expanded = 1;
    let mut best_generation = 0;

    for generation in 0..5 {
        let beam_width = 1 << (4 + generation); // 16, 32, 64, 128, 256
        trace!(generation, beam_width, total_expanded);
        beam.clear();
        beam.push(root);

        loop {
            if beam.len() > beam_width {
                beam.select_nth_unstable_by_key(beam_width, |n| n.ord_min_best());
                beam.truncate(beam_width);
            }

            next_beam.clear();
            for &node in beam.iter() {
                if node.children().is_empty() {
                    total_expanded += node.expand(&alo).len();
                }
                next_beam.extend_from_slice(node.children());
            }

            if next_beam.is_empty() {
                for &node in beam.iter() {
                    if node.ord_max_best() > best.ord_max_best() {
                        best = node;
                        best_generation = generation;
                        trace!(new_best = best.score(), generation, total_expanded);
                    }
                }
                break;
            }

            swap(&mut beam, &mut next_beam);
        }
    }

    let target = best.original_piece()?;

    debug!(best = best.score(), ?target, state = ?best.state());
    debug!(total_expanded, best_generation);
    trace!(alo_kb = alo.allocated_bytes() / 1024);

    let hold = target.piece != current;
    let reach_inputs = reach(matrix, target)?;
    Some((hold, reach_inputs))
}
