use crate::{traits::Decompress, CVec};
use bitpacking::{BitPacker, BitPacker8x};
use std::mem;

/// A trait defining functionality for buffered reading of a collection. This reduces en/decode
/// operations on a CVec value
pub trait BufferedCVec {
    /// Should return the buffer to use
    fn get_buffer(&self) -> &Vec<u32>;

    /// Should return the buffer to use mutable
    fn get_buffer_mut(&mut self) -> &mut Vec<u32>;

    /// Should set the buffer to `data`
    fn set_bufffer(&mut self, data: Vec<u32>);

    /// Should return the block index of the block stored in buffer
    fn get_buf_block(&self) -> Option<usize>;

    /// Should set the block index of the block stored in buffer
    fn set_buf_block(&mut self, pos: usize);

    /// Should return the CVec reference
    fn get_vec(&self) -> &CVec;

    /// Like CVec::get() but returns a reference to the u32 and uses a cache if available
    #[inline]
    fn get_buffered(&mut self, index: usize) -> Option<&u32> {
        if index >= self.get_vec().len() {
            return None;
        }

        let block_index = CVec::pos_block(index);

        let buf_block = self.get_buf_block();
        if buf_block.is_none() || buf_block.unwrap() != block_index {
            // Set cache

            let mut buff = mem::take(self.get_buffer_mut());
            self.get_vec().decompress_block(block_index, &mut buff);
            self.set_bufffer(buff);
            self.set_buf_block(block_index);
        }

        self.get_buffer().get(CVec::pos_in_block(index))
    }
}

/// A wrapper around an owned [`CVec`], which allows reading nearby indices faster
#[derive(Debug, Clone)]
pub struct BufCVec {
    vec: CVec,
    buf: Vec<u32>,
    buf_block: Option<usize>,
}

impl BufCVec {
    /// Create a new BufCVec from an owned CVec
    #[inline]
    pub fn new(vec: CVec) -> Self {
        Self {
            vec,
            buf: vec![0u32; BitPacker8x::BLOCK_LEN],
            buf_block: None,
        }
    }
}

impl From<CVec> for BufCVec {
    #[inline]
    fn from(cvec: CVec) -> Self {
        Self::new(cvec)
    }
}

/// A wrapper around a borrowed [`CVec`], which allows reading nearby indices faster
#[derive(Debug, Clone)]
pub struct BufCVecRef<'a> {
    vec: &'a CVec,
    buf: Vec<u32>,
    buf_block: Option<usize>,
}

impl<'a> From<&'a CVec> for BufCVecRef<'a> {
    #[inline]
    fn from(cvec: &'a CVec) -> Self {
        BufCVecRef::new(cvec)
    }
}

impl<'a> BufCVecRef<'a> {
    /// Create a new BufCVecRef from a CVec reference
    #[inline]
    pub fn new(vec: &'a CVec) -> Self {
        Self {
            vec,
            buf: vec![0u32; BitPacker8x::BLOCK_LEN],
            buf_block: None,
        }
    }
}

impl BufferedCVec for BufCVec {
    #[inline]
    fn get_buffer(&self) -> &Vec<u32> {
        &self.buf
    }

    #[inline]
    fn set_bufffer(&mut self, data: Vec<u32>) {
        self.buf = data;
    }

    #[inline]
    fn get_buf_block(&self) -> Option<usize> {
        self.buf_block
    }

    #[inline]
    fn set_buf_block(&mut self, buf_block: usize) {
        self.buf_block = Some(buf_block);
    }

    #[inline]
    fn get_vec(&self) -> &CVec {
        &self.vec
    }

    #[inline]
    fn get_buffer_mut(&mut self) -> &mut Vec<u32> {
        &mut self.buf
    }
}

impl<'a> BufferedCVec for BufCVecRef<'a> {
    #[inline]
    fn get_buffer(&self) -> &Vec<u32> {
        &self.buf
    }

    #[inline]
    fn set_bufffer(&mut self, data: Vec<u32>) {
        self.buf = data;
    }

    #[inline]
    fn get_buf_block(&self) -> Option<usize> {
        self.buf_block
    }

    #[inline]
    fn set_buf_block(&mut self, buf_block: usize) {
        self.buf_block = Some(buf_block);
    }

    #[inline]
    fn get_vec(&self) -> &CVec {
        &self.vec
    }

    #[inline]
    fn get_buffer_mut(&mut self) -> &mut Vec<u32> {
        &mut self.buf
    }
}
