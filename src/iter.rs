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

pub(crate) struct Chunked<T, U>
where
    T: Iterator<Item = U>,
    U: Clone,
{
    iter: T,
    size: usize,
    buf: Vec<T::Item>,
}

impl<T, U> Chunked<T, U>
where
    T: Iterator<Item = U>,
    U: Clone,
{
    #[inline]
    pub(crate) fn new(iter: T, size: usize) -> Self {
        Self {
            iter,
            size,
            buf: Vec::with_capacity(size),
        }
    }
}

pub(crate) trait Chunkable<U: Clone>: Iterator<Item = U> + Sized {
    #[inline]
    fn chunked(self, size: usize) -> Chunked<Self, Self::Item> {
        assert!(size > 0);
        Chunked::new(self, size)
    }
}

impl<T, U> Chunkable<U> for T
where
    T: Iterator<Item = U> + Sized,
    U: Clone,
{
}

impl<U: Clone, T: Iterator<Item = U>> Iterator for Chunked<T, U> {
    type Item = Vec<T::Item>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.iter.next();
            let empty = next.is_none();

            if empty && self.buf.is_empty() {
                return None;
            }

            if let Some(next) = next {
                self.buf.push(next);
            }

            if self.buf.len() >= self.size || empty {
                let res = Some(self.buf.clone());
                self.buf.clear();
                return res;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn chunked_iter() {
        let mut iter = (0..10).chunked(2);
        assert_eq!(iter.next(), Some(vec![0, 1]));
        assert_eq!(iter.next(), Some(vec![2, 3]));
        assert_eq!(iter.next(), Some(vec![4, 5]));
        assert_eq!(iter.next(), Some(vec![6, 7]));
        assert_eq!(iter.next(), Some(vec![8, 9]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn chunked_iter_empty() {
        let mut iter = std::iter::from_fn::<String, _>(|| None).chunked(1);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn chunked_non_align() {
        let mut iter = (0..11).chunked(2);
        assert_eq!(iter.next(), Some(vec![0, 1]));
        assert_eq!(iter.next(), Some(vec![2, 3]));
        assert_eq!(iter.next(), Some(vec![4, 5]));
        assert_eq!(iter.next(), Some(vec![6, 7]));
        assert_eq!(iter.next(), Some(vec![8, 9]));
        assert_eq!(iter.next(), Some(vec![10]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn chunked_size_larger() {
        let mut iter = (0..11).chunked(20);
        assert_eq!(iter.next(), Some((0..11).collect::<Vec<_>>()));
        assert_eq!(iter.next(), None);
    }
}
