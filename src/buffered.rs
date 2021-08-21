use crate::CVec;
use bitpacking::{BitPacker, BitPacker8x};
use std::mem;

/// A trait defining functionality for buffered reading of a collection. This reduces en/decode
/// operations on a CVec value
pub trait BufferedCVec {
    fn get_buffer(&mut self) -> &mut Buffer;

    /// Should return the CVec reference
    fn get_vec(&self) -> &CVec;

    /// Like CVec::get() but returns a reference to the u32 and uses a cache if available
    fn get_buffered(&mut self, index: usize) -> Option<&u32>;
}

/// A buffer for reading a [`CVec`] sequencially efficiently.
#[derive(Debug, Clone)]
pub struct Buffer {
    data: Vec<u32>,
    buf_block: Option<usize>,
}

impl Buffer {
    /// Create a new buffer with empty data preallocated
    #[inline]
    pub fn new() -> Self {
        Self {
            data: vec![],
            //data: vec![0u32; BitPacker8x::BLOCK_LEN],
            buf_block: None,
        }
    }

    pub fn read_buffered(&mut self, vec: &CVec, index: usize) -> Option<&u32> {
        if index >= vec.len() {
            return None;
        }

        let block_index = CVec::pos_block(index);

        if self.buf_block.is_none() || *self.buf_block.as_ref().unwrap() != block_index {
            // Set cache
            let mut buff = mem::take(&mut self.data);
            if self.data.len() < BitPacker8x::BLOCK_LEN {
                self.data.resize(BitPacker8x::BLOCK_LEN, 0);
            }
            vec.decompress_block(block_index, &mut buff);
            self.data = buff;
            self.buf_block = Some(block_index);
        }

        self.data.get(CVec::pos_in_block(index))
    }
}

/// A wrapper around an owned [`CVec`], which allows reading nearby indices faster
#[derive(Debug, Clone)]
pub struct BufCVec {
    vec: CVec,
    buf: Buffer,
}

impl BufCVec {
    /// Create a new BufCVec from an owned CVec
    #[inline]
    pub fn new(vec: CVec) -> Self {
        Self {
            vec,
            buf: Buffer::new(),
        }
    }

    /// Read from a `BufCVec`
    #[inline]
    pub fn get_buffered(&mut self, index: usize) -> Option<&u32> {
        self.buf.read_buffered(&self.vec, index)
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
    buf: Buffer,
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
            buf: Buffer::new(),
        }
    }

    #[inline]
    pub fn get_buffered(&mut self, index: usize) -> Option<&u32> {
        self.buf.read_buffered(&self.vec, index)
    }
}

impl BufferedCVec for BufCVec {
    #[inline]
    fn get_buffer(&mut self) -> &mut Buffer {
        &mut self.buf
    }

    #[inline]
    fn get_vec(&self) -> &CVec {
        &self.vec
    }

    #[inline]
    fn get_buffered(&mut self, index: usize) -> Option<&u32> {
        self.get_buffered(index)
    }
}

impl<'a> BufferedCVec for BufCVecRef<'a> {
    #[inline]
    fn get_buffer(&mut self) -> &mut Buffer {
        &mut self.buf
    }

    #[inline]
    fn get_vec(&self) -> &CVec {
        &self.vec
    }

    #[inline]
    fn get_buffered(&mut self, index: usize) -> Option<&u32> {
        self.get_buffered(index)
    }
}
