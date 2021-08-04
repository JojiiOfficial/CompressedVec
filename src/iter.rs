use std::iter::FromIterator;

use crate::{
    buffered::{BufCVec, BufCVecRef, BufferedCVec},
    CVec,
};

/// Helper type to iterate a borrowed CVec
pub struct CVecIterRef<'a> {
    vec: BufCVecRef<'a>,
    pos: usize,
}

impl<'a> CVecIterRef<'a> {
    #[inline]
    pub(crate) fn new(vec: &'a CVec) -> Self {
        Self {
            vec: BufCVecRef::new(vec),
            pos: 0,
        }
    }
}

impl<'a> Iterator for CVecIterRef<'a> {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let val = self.vec.get_buffered(self.pos)?;
        self.pos += 1;
        Some(*val)
    }
}

/// Helper type to iterate a CVec
pub struct CVecIter {
    vec: BufCVec,
    pos: usize,
}

impl Iterator for CVecIter {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let val = self.vec.get_buffered(self.pos)?;
        self.pos += 1;
        Some(*val)
    }
}

impl IntoIterator for CVec {
    type Item = u32;

    type IntoIter = CVecIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        CVecIter {
            vec: BufCVec::new(self),
            pos: 0,
        }
    }
}

impl FromIterator<u32> for CVec {
    #[inline]
    fn from_iter<T: IntoIterator<Item = u32>>(iter: T) -> Self {
        let mut new = CVec::new();

        // TODO make this faster. This can be done by implementing some sort of .extend()
        // function first
        for i in iter {
            new.push(i);
        }

        new
    }
}

impl<T: AsRef<u32>> From<Vec<T>> for CVec {
    #[inline]
    fn from(vec: Vec<T>) -> Self {
        vec.into_iter().map(|i| *i.as_ref()).collect::<Self>()
    }
}
