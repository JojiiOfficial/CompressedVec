use std::iter::FromIterator;

use crate::{
    buffered::{BufCVec, BufCVecRef, BufferedCVec},
    CVec,
};

/// `Iterator` implementing type to iterate over a `&CVec`
pub struct CVecIterRef<'a> {
    vec: BufCVecRef<'a>,
    pos: usize,
    len: usize,
}

impl<'a> CVecIterRef<'a> {
    #[inline]
    pub(crate) fn new(vec: &'a CVec) -> Self {
        Self {
            vec: BufCVecRef::new(vec),
            pos: 0,
            len: vec.len(),
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

/// `Iterator` implementing type to iterate over a `CVec`
pub struct CVecIter {
    vec: BufCVec,
    pos: usize,
    len: usize,
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
            len: self.len(),
            vec: BufCVec::new(self),
            pos: 0,
        }
    }
}

impl FromIterator<u32> for CVec {
    #[inline]
    fn from_iter<T: IntoIterator<Item = u32>>(iter: T) -> Self {
        let mut new = CVec::new();
        new.extend(iter);
        new
    }
}

impl ExactSizeIterator for CVecIter {
    #[inline]
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a> ExactSizeIterator for CVecIterRef<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.len
    }
}
