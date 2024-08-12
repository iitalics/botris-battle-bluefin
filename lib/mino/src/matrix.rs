//! Matrix data structure.

use alloc::vec::Vec;
use core::mem::transmute;
use core::ops::Deref;

use crate::piece::Cells;

/// Matrix representation. Represented as slice of `u16` per row, with individual bits
/// containing column data.
pub struct Mat([u16]);

pub const COLS: i8 = 10;
pub const FULL: u16 = !0;
pub const EMPTY: u16 = FULL << COLS;

impl Mat {
    pub const fn empty() -> &'static Mat {
        Self::new(&[])
    }

    pub const fn new(slice: &[u16]) -> &Mat {
        debug_assert!(slice.len() < i8::MAX as usize);
        unsafe { transmute(slice) }
    }

    pub const fn rows(&self) -> &[u16] {
        &self.0
    }

    pub const fn cols(&self) -> i8 {
        COLS
    }

    pub const fn len(&self) -> i8 {
        self.rows().len() as i8
    }

    /// # Safety
    ///
    /// - `y` must be within bounds (0 <= y <= len()).
    pub unsafe fn get_unchecked(&self, y: i8) -> u16 {
        *self.rows().get_unchecked(y as usize)
    }

    pub fn get(&self, y: i8) -> u16 {
        if let Ok(y) = usize::try_from(y) {
            if let Some(row) = self.rows().get(y) {
                *row
            } else {
                // y >= len
                EMPTY
            }
        } else {
            // y < 0
            FULL
        }
    }
}

/// Mutable matrix, can add rows and set bits on existing rows.
#[derive(Clone)]
pub struct MatBuf(Vec<u16>);

impl MatBuf {
    /// Allocate a new mutable matrix, initially empty.
    pub fn new() -> Self {
        Self(Vec::with_capacity(20))
    }

    fn rows_mut(&mut self) -> &mut Vec<u16> {
        &mut self.0
    }

    /// Clear the matrix so that it is empty again.
    pub fn clear(&mut self) {
        self.rows_mut().clear();
    }

    /// Set this matrix to be identical to the given matrix.
    pub fn copy_from(&mut self, mat: &Mat) {
        self.rows_mut().clear();
        self.rows_mut().extend_from_slice(mat.rows());
    }

    /// Set the column bits for the given row.
    pub fn set(&mut self, y: i8, bits: u16) {
        let rows = self.rows_mut();
        if let Ok(y) = usize::try_from(y) {
            if rows.len() <= y {
                rows.resize(y + 1, EMPTY);
            }
            rows[y] |= bits;
        }
    }

    /// Places the cells onto the matrix by filling in the occupied coordinates.
    pub fn place(&mut self, cells: Cells) {
        cells.place(self)
    }

    /// Remove rows that are full above row `y_start`, moving any rows above them into
    /// their place. Returns the number of rows removed.
    pub fn clear_lines(&mut self, y_start: i8) -> u8 {
        let rows = self.rows_mut();
        let y_end = rows.len();
        let y_start = usize::try_from(y_start).unwrap_or(0).min(y_end);
        let mut y_to = y_start;
        for y in y_start..y_end {
            if rows[y] != FULL {
                rows[y_to] = rows[y];
                y_to += 1;
            }
        }
        // SAFETY: 0 <= y_start <= y_to < y_end
        unsafe { rows.set_len(y_to) };
        y_end as u8 - y_to as u8
    }
}

impl Default for MatBuf {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for MatBuf {
    type Target = Mat;
    fn deref(&self) -> &Mat {
        Mat::new(&self.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_empty_bits() {
        for x in 0..13 {
            let occ = (EMPTY & (1 << x)) != 0;
            assert_eq!((x, occ), (x, x >= COLS));
        }
    }

    #[test]
    fn test_get_empty() {
        let mat = Mat::empty();
        assert_eq!(mat.get(0), EMPTY);
        assert_eq!(mat.get(1), EMPTY);
        assert_eq!(mat.get(-1), FULL);
    }

    #[test]
    fn test_mat_set() {
        let mut mat = MatBuf::new();
        mat.set(0, 0b1);
        assert_eq!(mat.get(0), EMPTY | 0b1);
        assert_eq!(mat.get(1), EMPTY);
        mat.set(2, 0b100);
        assert_eq!(mat.get(0), EMPTY | 0b1);
        assert_eq!(mat.get(1), EMPTY);
        assert_eq!(mat.get(2), EMPTY | 0b100);
        assert_eq!(mat.get(3), EMPTY);
        mat.set(0, 0b110000);
        assert_eq!(mat.get(0), EMPTY | 0b110001);
    }

    #[test]
    fn test_mat_clear_lines() {
        let mut mat = MatBuf::new();
        assert_eq!(mat.clear_lines(0), 0);
        assert_eq!(mat.len(), 0);
        mat.set(0, FULL);
        mat.set(1, FULL);
        mat.set(2, 0b100);
        mat.set(3, FULL);
        mat.set(4, FULL);
        assert_eq!(mat.clear_lines(1), 3);
        assert_eq!(mat.len(), 2);
        assert_eq!(mat.get(0), FULL, "{:b}", mat.get(0));
        assert_eq!(mat.get(1), EMPTY | 0b100, "{:b}", mat.get(1));
        assert_eq!(mat.get(2), EMPTY, "{:b}", mat.get(2));
        assert_eq!(mat.clear_lines(1), 0);
        assert_eq!(mat.len(), 2);
        assert_eq!(mat.clear_lines(0), 1);
        assert_eq!(mat.len(), 1);
        assert_eq!(mat.get(0), EMPTY | 0b100, "{:b}", mat.get(0));
        assert_eq!(mat.get(1), EMPTY, "{:b}", mat.get(1));
    }
}
