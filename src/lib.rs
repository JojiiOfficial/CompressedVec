use std::mem::size_of;

use bitpacking::{BitPacker, BitPacker8x};

/// A compressed Vec<u32>
#[derive(Clone, Debug)]
pub struct CVec {
    /// The compressed Data
    data: Vec<(u8, Vec<u8>)>,
    /// Count of items in the vector
    items: usize,
    /// Length of compressed data in last element
    last_compr_len: usize,
}

impl CVec {
    #[inline]
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            items: 0,
            last_compr_len: 0,
        }
    }

    /// Returns the amount of allocated bytes by the vector
    #[inline]
    pub fn byte_len(&self) -> usize {
        // `items` and `last_compr_len`
        (size_of::<usize>() * 2)
            // data.len() * (u8, Vec<u8>)
            + (self.data.len()
                * (size_of::<usize>() + 256 + 1))
            // Initial data vector
            + size_of::<usize>()
    }

    /// Returns the amount of items stored in the vector
    #[inline]
    pub fn len(&self) -> usize {
        self.items
    }

    /// Returns true if the vector is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Pushes a new value on top of the vector
    pub fn push(&mut self, val: u32) {
        if self.is_empty() || self.need_new_block() {
            let (enc, num_bits, len) = Self::compress_block(vec![val]);
            self.last_compr_len = len;
            self.data.push((num_bits, enc));
        } else {
            // decompress last block
            let mut block = self.decompress_block(self.data.len() - 1).unwrap();
            // Set value at position
            block[self.items % 256] = val;

            // Compress block again
            let (enc, num_bits, len) = Self::compress_block(block);
            *self.data.last_mut().unwrap() = (num_bits, enc);
            self.last_compr_len = len;
        }

        self.items += 1;
    }

    /// Returns the u32 at `pos`
    #[inline]
    pub fn get(&self, pos: usize) -> Option<u32> {
        if pos > self.items {
            return None;
        }
        self.decompress_block(pos / 256)?.get(pos % 256).map(|i| *i)
    }

    /// Compresse a Vec<u32>
    ///
    /// # Panics
    /// Panics if data.len() > 256
    fn compress_block(mut data: Vec<u32>) -> (Vec<u8>, u8, usize) {
        assert!(data.len() <= 256);

        if data.len() < 256 {
            data.extend((0..(256 - data.len() as u32 % 256)).map(|_| 0));
        }

        let bitpacker = BitPacker8x::new();
        let num_bits: u8 = bitpacker.num_bits(&data);
        let mut compressed = vec![0u8; 4 * BitPacker8x::BLOCK_LEN];
        let compressed_len = bitpacker.compress(&data, &mut compressed[..], num_bits);
        (compressed, num_bits, compressed_len)
    }

    /// Decompress a given block
    fn decompress_block(&self, index: usize) -> Option<Vec<u32>> {
        let bitpacker = BitPacker8x::new();
        let (num_bits, block) = self.data.get(index)?;
        let mut decompressed = vec![0u32; BitPacker8x::BLOCK_LEN];

        let compressed_len = if index == self.data.len() - 1 {
            self.last_compr_len
        } else {
            (*num_bits as usize) * BitPacker8x::BLOCK_LEN / 8
        };

        bitpacker.decompress(&block[..compressed_len], &mut decompressed[..], *num_bits);

        Some(decompressed)
    }

    /// Returns true if a new block needs to be allocated.
    #[inline]
    fn need_new_block(&self) -> bool {
        (self.is_empty() && self.data.is_empty()) || self.items % 256 == 0
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_alloc_new() {
        let mut v = CVec::new();
        assert!(v.need_new_block());
        v.push(1);
        assert!(!v.need_new_block());
    }

    #[test]
    fn test_push() {
        let test_data = (0..99999).collect::<Vec<_>>();

        let mut v = CVec::new();
        for i in test_data.iter() {
            v.push(*i);
        }
        assert_eq!(v.len(), test_data.len());

        for (pos, i) in test_data.iter().enumerate() {
            assert_eq!(v.get(pos).unwrap(), *i);
        }
    }
}
