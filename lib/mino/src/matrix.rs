//! Matrix data structure.

use alloc::vec::Vec;
use core::mem::transmute;
use core::ops::Deref;

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
    pub fn new() -> Self {
        Self(Vec::with_capacity(20))
    }

    fn rows_mut(&mut self) -> &mut Vec<u16> {
        &mut self.0
    }

    pub fn clear(&mut self) {
        self.rows_mut().clear();
    }

    pub fn set(&mut self, y: i8, bits: u16) {
        if let Ok(y) = usize::try_from(y) {
            if self.rows().len() <= y {
                self.rows_mut().resize(y + 1, EMPTY);
            }
            self.rows_mut()[y] |= bits;
        }
    }

    pub fn copy_from(&mut self, mat: &Mat) {
        self.rows_mut().clear();
        self.rows_mut().extend_from_slice(mat.rows());
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
        assert_eq!(mat.get(1), EMPTY | 0b0);
        assert_eq!(mat.get(2), EMPTY | 0b100);
        assert_eq!(mat.get(3), EMPTY);
        mat.set(0, 0b110000);
        assert_eq!(mat.get(0), EMPTY | 0b110001);
    }
}
