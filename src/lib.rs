//! A simple crate which provides a compressed List of u32 values. The level of compression is
//! dependent on the size of the pushed numbers and can be up to 32 times in size which is the case
//! for bit sequences.

/// Contains a ro-wrapper around `CVec` that caches read blocks for faster sequencial (or nearby)
/// access to the `CVec` values.
pub mod buffered;
/// Contains iterator implementations for `CVec`
pub mod iter;
pub mod traits;

pub use buffered::Buffer;

use bitpacking::{BitPacker, BitPacker8x};
use iter::CVecIterRef;
use serde::{Deserialize, Serialize};
use std::mem::size_of;
use utilsrs::itertools::IterExt;

/// A compressed `Vec<u32>` which can be compress up to 32 times in size. The level of compression
/// depends on the bitsize of the biggest value within a 256block.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CVec {
    /// The compressed Data
    data: Vec<(u8, Vec<u8>)>,

    /// Count of items in the vector
    items: usize,
}

impl CVec {
    /// Constructs a new, empty `CVec`
    #[inline]
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            items: 0,
        }
    }

    /// Allocate a new compressed vector which can store `capacity` numbers without reallocating
    pub fn with_capacity(capacity: usize) -> Self {
        let req_blocks = Self::req_block_count(capacity);

        let data = (0..req_blocks)
            .map(|_| (0, Vec::with_capacity(256)))
            .collect();

        Self { data, items: 0 }
    }

    /// Returns the amount of allocated bytes by the vector
    #[inline]
    pub fn byte_len(&self) -> usize {
        // `items` and initial `data` vec
        let mut len = size_of::<usize>() * 2;

        for block in self.data.iter() {
            // u8 size
            len += 1;
            // block  size
            len += block.1.len();
        }

        len
    }

    /// Returns the number of elements in the vector
    #[inline]
    pub fn len(&self) -> usize {
        self.items
    }

    /// Returns true if the vector is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of numbers the vector can hold without reallocating
    #[inline]
    pub fn capacity(&self) -> usize {
        self.data.len() * 256
    }

    /// Pushes a new value on top of the vector
    pub fn push(&mut self, val: u32) {
        if self.need_new_block() {
            let mut new_block = Vec::with_capacity(256);
            let num_bits = Self::compress(vec![val], &mut new_block);
            self.data.push((num_bits, new_block));
        } else {
            let block_nr = self.last_block();

            // decompress last block
            let mut block = vec![0u32; BitPacker8x::BLOCK_LEN];
            self.decompress_block(block_nr, &mut block).unwrap();

            // Set value at position
            block[self.items % 256] = val;

            // If get_mut would return None, the if block was executed.
            let mut out_block = self.data.get_mut(block_nr).unwrap();

            // Compress block again
            let bit_size = Self::compress(block, &mut out_block.1);
            out_block.0 = bit_size;
        }

        self.items += 1;
    }

    /// Pops the last element from the vector. Returns `None` if vector is empty or Some(val)
    /// with the popped value
    pub fn pop(&mut self) -> Option<u32> {
        if self.is_empty() {
            return None;
        }

        let popped = self.last_unchecked()?;

        self.items -= 1;

        // Remove last allocated block if it gets empty
        if self.items % 256 == 0 {
            let block_nr = self.last_block();
            self.data.remove(block_nr);
        }

        Some(popped)
    }

    /// Returns the last number in the vector. `None` if `self.len() == 0`
    #[inline]
    pub fn last(&self) -> Option<u32> {
        if self.is_empty() {
            return None;
        }

        self.last_unchecked()
    }

    /// Returns the u32 at `pos`
    pub fn get(&self, pos: usize) -> Option<u32> {
        if pos >= self.items {
            return None;
        }

        let mut decompressed = vec![0u32; BitPacker8x::BLOCK_LEN];
        self.decompress_block(Self::pos_block(pos), &mut decompressed)?;
        decompressed.get(Self::pos_in_block(pos)).map(|i| *i)
    }

    /// Returns the u32 at `pos`
    pub fn set(&mut self, pos: usize, new: u32) -> Option<()> {
        if pos >= self.items {
            return None;
        }

        let mut decompressed = vec![0u32; BitPacker8x::BLOCK_LEN];
        self.decompress_block(Self::pos_block(pos), &mut decompressed)?;
        *decompressed.get_mut(Self::pos_in_block(pos))? = new;
        let bit_size = Self::compress(
            decompressed,
            &mut self.data.get_mut(Self::pos_block(pos)).unwrap().1,
        );
        self.data.get_mut(Self::pos_block(pos)).unwrap().0 = bit_size;

        Some(())
    }

    /// Returns an referenced iterator over the vector's elements
    #[inline]
    pub fn iter<'a>(&'a self) -> CVecIterRef<'a> {
        CVecIterRef::new(self)
    }

    /// Returns the data hold by CVec decompressed as `Vec::<u32>`
    #[inline]
    pub fn as_vec(&self) -> Vec<u32> {
        Vec::from(self)
    }

    /// Returns the block `pos` is stored in
    #[inline]
    pub(crate) fn pos_block(pos: usize) -> usize {
        pos / 256
    }

    /// Returns the position of `pos` in a block
    #[inline]
    pub(crate) fn pos_in_block(pos: usize) -> usize {
        pos % 256
    }

    /// Returns the index in `self.data` of the last block
    #[inline]
    pub(crate) fn last_block(&self) -> usize {
        Self::pos_block(self.items)
    }

    /// Returns true if a new block needs to be allocated.
    #[inline]
    fn need_new_block(&self) -> bool {
        self.items / 256 >= self.data.len()
    }

    /// Returns the amount of blocks required to store `size` elements
    #[inline]
    fn req_block_count(size: usize) -> usize {
        if size % 256 != 0 {
            Self::pos_block(size) + 1
        } else {
            Self::pos_block(size)
        }
    }

    /// Returns the last number in the vector. `None` if `self.len() == 0` without doing safety
    /// checks.
    #[inline]
    fn last_unchecked(&self) -> Option<u32> {
        self.get(self.len() - 1)
    }

    /// Compresses a Vec<u32>
    ///
    /// # Panics
    /// Panics if data.len() > 256
    fn compress(mut data: Vec<u32>, out: &mut Vec<u8>) -> u8 {
        assert!(data.len() <= 256);

        if data.len() < 256 {
            data.extend((0..(256 - data.len() as u32 % 256)).map(|_| 0));
        }

        let bitpacker = BitPacker8x::new();
        let num_bits: u8 = bitpacker.num_bits(&data);

        let out_size = 32 * num_bits as usize;
        out.resize(out_size, 0);

        bitpacker.compress(&data, out, num_bits);
        num_bits
    }

    /// Decompress a given block at `index`
    ///
    /// Returns `None` if there is no such block.
    #[inline]
    fn decompress_block(&self, index: usize, out: &mut Vec<u32>) -> Option<()> {
        let (num_bits, block) = self.data.get(index)?;
        Self::decompress(block, *num_bits, out);
        Some(())
    }

    /// Decompresses `data` and writes them to `out`. If `out` has an invalid size, it gets padded
    /// with 0s.
    ///
    /// # Panics
    /// panics if `data` is too short
    fn decompress(data: &[u8], num_bits: u8, out: &mut Vec<u32>) {
        let bitpacker = BitPacker8x::new();

        if out.len() < BitPacker8x::BLOCK_LEN {
            out.resize(BitPacker8x::BLOCK_LEN, 0);
        }

        let compressed_len = (num_bits as usize) * BitPacker8x::BLOCK_LEN / 8;
        bitpacker.decompress(
            &data[..compressed_len],
            &mut out[0..BitPacker8x::BLOCK_LEN],
            num_bits,
        );
    }
}

impl Extend<u32> for CVec {
    /// Reads all values from `iter` and pushes them onto the vector. This should be preferred over
    /// `push` if you have more than one value to append.
    fn extend<T: IntoIterator<Item = u32>>(&mut self, iter: T) {
        let mut iter = iter.into_iter();

        // How many items were pushed
        let mut pushed: usize = 0;

        // Fill last block
        if !self.need_new_block() {
            let last_block_idx = self.last_block();

            let free_slots = 256 - (self.items % 256);
            let to_fill = free_slots;

            // decompress last block
            let mut block = vec![0u32; BitPacker8x::BLOCK_LEN];
            self.decompress_block(last_block_idx, &mut block).unwrap();

            // Set all values
            let start = self.items % 256;
            for i in start..start + to_fill {
                block[i] = match iter.next() {
                    Some(s) => s,
                    None => break,
                };
                pushed += 1;
            }

            // Compress block again
            let mut out_block = self.data.get_mut(last_block_idx).unwrap();
            let bit_size = Self::compress(block, &mut out_block.1);
            out_block.0 = bit_size;
            self.items += pushed;
        }

        // Push rest of `iter` into new block(s)
        let mut block = Vec::new();
        for to_add in iter.by_ref().chunked(256) {
            self.items += to_add.len();
            let num_bits = Self::compress(to_add, &mut block);
            self.data.push((num_bits, block.clone()));
        }
    }
}
