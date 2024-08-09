//! Data structures for queue manipulation.

use core::{fmt, mem};

/// Represents the upcoming pieces in the queue, incl. held piece.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Queue<'a, T> {
    hold: Option<T>,
    next: &'a [T],
}

impl<'a, T: Copy> Queue<'a, T> {
    /// Constructs a new queue from the hold and next pieces. If `hold` is empty (`None`),
    /// then this will automatically try to take a piece from `next` and put it into the
    /// hold.
    pub const fn new(hold: Option<T>, next: &'a [T]) -> Self {
        match (hold, next) {
            (None, [front, back @ ..]) => Self {
                hold: Some(*front),
                next: back,
            },
            (_, _) => Self { hold, next },
        }
    }

    /// Get the held piece.
    pub fn hold(&self) -> Option<T> {
        self.hold
    }

    /// Get the next pieces.
    pub fn next(&self) -> &'a [T] {
        self.next
    }

    /// Returns an iterator of `(piece, new_queue)`, where `piece` is an immediately
    /// reachable piece (maybe requiring hold), and `new_queue` is the queue afterwards.
    pub fn pop(&self) -> Pop<'a, T> {
        Pop {
            did_hold: false,
            curr: self.hold,
            succ: self.next.into(),
        }
    }
}

impl<'a, T: Copy> From<&'a [T]> for Queue<'a, T> {
    fn from(pieces: &'a [T]) -> Self {
        Self::new(None, pieces)
    }
}

impl<'a, T: Copy, const N: usize> From<&'a [T; N]> for Queue<'a, T> {
    fn from(pieces: &'a [T; N]) -> Self {
        pieces[..].into()
    }
}

impl<T: fmt::Display> fmt::Display for Queue<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (hold, curr, next) = match &self.next {
            [front, back @ ..] => (self.hold.as_ref(), Some(front), back),
            [] => (None, self.hold.as_ref(), &[][..]),
        };
        f.write_str("[")?;
        if let Some(x) = hold {
            x.fmt(f)?;
        }
        f.write_str("](")?;
        if let Some(x) = curr {
            x.fmt(f)?;
        }
        f.write_str(")")?;
        for x in next {
            x.fmt(f)?;
        }
        Ok(())
    }
}

pub struct Pop<'a, T: Copy> {
    did_hold: bool,
    curr: Option<T>,
    succ: Queue<'a, T>,
}

impl<'a, T: Copy> Iterator for Pop<'a, T> {
    type Item = (T, Queue<'a, T>);

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.curr?;
        let succ = self.succ;
        if self.did_hold {
            self.curr = None;
        } else {
            self.did_hold = true;
            mem::swap(&mut self.curr, &mut self.succ.hold);
        }
        Some((curr, succ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_queue_hold_next() {
        let q = Queue::from(b"LOJT");
        assert_eq!(q.hold(), Some(b'L'));
        assert_eq!(q.next(), b"OJT");
        let q = Queue::from(b"");
        assert_eq!(q.hold(), None);
        assert_eq!(q.next(), b"");
    }

    #[test]
    fn test_queue_pop() {
        let q = Queue::from(b"LOJT");
        let mut qs = q.pop();
        let (x1, q1) = qs.next().unwrap();
        assert_eq!(x1, b'L');
        assert_eq!(q1, Queue::from(b"OJT"));
        let (x2, q2) = qs.next().unwrap();
        assert_eq!(x2, b'O');
        assert_eq!(q2, Queue::from(b"LJT"));
        assert!(qs.next().is_none());
    }

    /*
    // not valid
    #[test]
    fn test_queue_same_front() {
        let q = Queue::from(b"LLJT");
        let mut qs = q.pop();
        let (x1, q1) = qs.next().unwrap();
        assert_eq!(x1, b'L');
        assert_eq!(q1, Queue::from(b"LJT"));
        assert!(qs.next().is_none());
    }
    */
}
