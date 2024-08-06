#![no_std]

extern crate alloc;

#[cfg(test)]
extern crate std;

pub mod matrix;
pub use matrix::{Mat, MatBuf};

pub mod input;
pub use input::{Dir, Input, Rot, Turn};

pub mod piece;
pub use piece::{Cells, Pos};

pub mod queue;
pub use queue::Queue;

pub mod places;
pub use places::{places, reach, Places};

pub mod game_state;
pub use game_state::GameState;

pub mod standard_rules;

#[cfg(test)]
mod test {
    use core::fmt;

    pub fn assert_same_set<T, XS, YS, W>(xs: XS, ys: YS, why: &W)
    where
        T: Ord + fmt::Debug,
        XS: IntoIterator<Item = T>,
        YS: IntoIterator<Item = T>,
        W: fmt::Display + ?Sized,
    {
        use alloc::collections::BTreeSet;
        use alloc::vec::Vec;
        let xs = xs.into_iter().collect::<BTreeSet<_>>();
        let ys = ys.into_iter().collect::<BTreeSet<_>>();
        let xs_ys = xs.difference(&ys).collect::<Vec<_>>();
        let ys_xs = ys.difference(&xs).collect::<Vec<_>>();
        assert!(
            xs_ys.is_empty() && ys_xs.is_empty(),
            "different sets: {why}\n  left: +{xs_ys:?}\n right: +{ys_xs:?}"
        );
    }
}
