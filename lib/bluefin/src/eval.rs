use mino::matrix;
use mino::matrix::Mat;

use crate::state::State;

pub mod weight {
    pub const BASE: i32 = 1024; // random number to make the values generally positive
    pub const SINGLE: i32 = -200;
    pub const DOUBLE: i32 = -100;
    pub const TRIPLE: i32 = 400;
    pub const QUAD: i32 = 1024; // "reference point"; do not change
    pub const SPIN_SINGLE: i32 = 512;
    pub const SPIN_DOUBLE: i32 = 1200;
    pub const SPIN_TRIPLE: i32 = 1600;
    pub const B2B: i32 = 200;
    // pub const B2B_BROKEN: i32
    pub const HEIGHT: i32 = -50;
    pub const ROW_TRANSITIONS: i32 = -200;
    pub const BLOCKS_FROM_TARGET: i32 = -20;
}

pub fn evaluate(mat: &Mat, st: State) -> i32 {
    let mut height = 0;
    let mut row_trans = 0;
    let mut block_count = 0;
    let blocks_from_target;

    {
        let mut prev = matrix::FULL;
        for &row in mat.rows() {
            row_trans += (row ^ prev).count_ones() as i32;
            block_count += row.count_ones() as i32;
            height += 1;
            prev = row;
        }
        row_trans += prev.count_ones() as i32 - 16;
        block_count -= height * 6;
        blocks_from_target = (block_count - 36).abs();
    }

    // trace!(height, row_trans, blocks_from_target);

    weight::BASE
        + st.single_clears as i32 * weight::SINGLE
        + st.double_clears as i32 * weight::DOUBLE
        + st.triple_clears as i32 * weight::TRIPLE
        + st.quad_clears as i32 * weight::QUAD
        + st.spin_single_clears as i32 * weight::SPIN_SINGLE
        + st.spin_double_clears as i32 * weight::SPIN_DOUBLE
        + st.spin_triple_clears as i32 * weight::SPIN_TRIPLE
        + st.b2b_clears as i32 * weight::B2B
        + height * weight::HEIGHT
        + row_trans * weight::ROW_TRANSITIONS
        + blocks_from_target * weight::BLOCKS_FROM_TARGET
}
