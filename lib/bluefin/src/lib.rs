#[macro_use]
extern crate tracing;

use mino::input::Input;
use mino::matrix::{Mat, MatBuf};
use mino::places::{places, reach};
use mino::standard_rules::{PieceType, Queue};

pub fn bot(
    current: PieceType,
    queue: &[PieceType],
    hold: Option<PieceType>,
    matrix: &Mat,
    // b2b: bool,
    // ren: u32,
    // jeopardy: [u32; 2],
) -> Option<(bool, Vec<Input>)> {
    let combined_queue: Vec<PieceType> = [hold.as_slice(), &[current], queue]
        .into_iter()
        .flat_map(|x| x.iter().copied())
        .collect();
    let queue = Queue::from(&*combined_queue);

    let mut new_matrix = MatBuf::new();
    let mut best = None;

    for (ty, _) in queue.pop() {
        for pc in places(matrix, ty) {
            new_matrix.clear();
            new_matrix.copy_from(matrix);
            let cells = pc.cells();
            cells.place(&mut new_matrix);
            let cleared = new_matrix.clear_lines(cells.extents().1.start);
            let score = evaluate(&new_matrix, cleared);
            best = best.max(Some((score, pc.piece)));
            if best.unwrap().1 == pc.piece {
                debug!("+{:?}: {}", pc.piece, score);
            } else {
                trace!("{:?}: {}", pc.piece, score);
            }
        }
    }

    let (score, best) = best?;
    debug!("best: {:?}: {}", best, score);
    let reach_inputs = reach(matrix, best)?;
    let hold = best.shape != current;
    Some((hold, reach_inputs))
}

fn evaluate(mat: &Mat, cleared: u8) -> i32 {
    let height = mat.len() as i32;
    let row_trans = row_transitions(mat);
    let target_blocks = (count_blocks(mat) - 36).abs();

    let mut score = 0;
    score -= 10 * height;
    score -= 60 * row_trans;
    score -= 2 * target_blocks;
    if cleared == 1 {
        score -= 30;
    }
    if cleared == 3 {
        score += 30;
    }
    if cleared == 4 {
        score += 100;
    }

    trace!(height, row_trans, target_blocks, score, "eval");
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
