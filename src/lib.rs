pub mod buffered;
pub mod iter;
pub mod traits;

use bitpacking::{BitPacker, BitPacker8x};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use iter::CVecIterRef;
use std::{
    cmp::min,
    io::{Cursor, Read, Write},
    mem::size_of,
};
use traits::{Compress, Decompress};

/// A compressed Vec<u32> which can be compress up to 4 times in size. The level of compression
/// depends on the actual value of each number pushed on the cvec.
#[derive(Clone, Debug)]
pub struct CVec {
    /// The compressed Data
    data: Vec<(u8, Vec<u8>)>,

    /// Count of items in the vector
    items: usize,
}

impl CVec {
    /// Allocate a new Compressed vector
    #[inline]
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            items: 0,
        }
    }

    /// Allocate a new compressed vector which can store `capacity` numbers without reallocating
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let req_blocks = Self::req_block_count(capacity);
        let mut data = Vec::with_capacity(req_blocks);

        for _ in 0..req_blocks {
            data.push((0, Vec::with_capacity(256)));
        }

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

    /// Returns the number of numbers the vector can hold without reallocating
    #[inline]
    pub fn capacity(&self) -> usize {
        self.data.len() * 256
    }

    /// Returns true if the vector is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Pushes a new value on top of the vector
    pub fn push(&mut self, val: u32) {
        if self.data.is_empty() || self.need_new_block() {
            let mut new_block = Vec::with_capacity(256);
            let num_bits = Self::compress_block(vec![val], &mut new_block);
            self.data.push((num_bits, new_block));
        } else {
            let block_nr = self.last_block();

            // decompress last block
            let mut block = vec![0u32; BitPacker8x::BLOCK_LEN];
            self.decompress_block(block_nr, &mut block).unwrap();

            // Set value at position
            block[self.items % 256] = val;

            let mut out_block = self.data.get_mut(block_nr).unwrap();

            // Compress block again
            let bit_size = Self::compress_block(block, &mut out_block.1);
            out_block.0 = bit_size;
        }

        self.items += 1;
    }

    /// Reads all values from `iter` and pushes them onto the vector. This should be preferred over
    /// `push` if you have more than one value you want to add.
    pub fn extend<T: ExactSizeIterator<Item = u32>>(&mut self, mut iter: T) {
        // TODO don't use exact sized iterator
        let items = iter.len();
        if items == 0 {
            return;
        }

        // How many items were pushed
        let mut pushed: usize = 0;

        // Fill last block
        if !self.need_new_block() {
            let last_block_idx = self.last_block();

            let free_slots = 256 - (self.items % 256);
            let to_fill = min(free_slots, items);

            // decompress last block
            let mut block = vec![0u32; BitPacker8x::BLOCK_LEN];
            self.decompress_block(last_block_idx, &mut block).unwrap();

            // Set all values
            let start = self.items % 256;
            for i in start..start + to_fill {
                block[i] = iter.next().unwrap();
                pushed += 1;
            }

            // Compress block again
            let mut out_block = self.data.get_mut(last_block_idx).unwrap();
            let bit_size = Self::compress_block(block, &mut out_block.1);
            out_block.0 = bit_size;
            self.items += to_fill;
        }

        // For the rest (if available) we're allocating new blocks which we fill and add
        if items > pushed {
            let to_push_left = items - pushed;

            let blocks_needed = (to_push_left / 256) + 1;
            let mut new_blocks = vec![(0u8, Vec::<u8>::new()); blocks_needed];

            for nb_pos in 0..blocks_needed {
                let block_data_raw = iter.by_ref().take(256).collect();
                new_blocks[nb_pos].0 =
                    Self::compress_block(block_data_raw, &mut new_blocks[nb_pos].1);
            }

            self.data.extend(new_blocks);
            self.items += to_push_left;
        }
    }

    /// Pops the last element from the vector. Returns `None` if vector is empty or Some(val)
    /// with the popped value
    pub fn pop(&mut self) -> Option<u32> {
        if self.is_empty() {
            return None;
        }

        let popped = self.last()?;

        self.items -= 1;

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

        self.get(self.len() - 1)
    }

    /// Returns the u32 at `pos`
    #[inline]
    pub fn get(&self, pos: usize) -> Option<u32> {
        if pos >= self.items {
            return None;
        }

        let mut decompressed = vec![0u32; BitPacker8x::BLOCK_LEN];

        self.decompress_block(Self::pos_block(pos), &mut decompressed)?;
        decompressed.get(Self::pos_in_block(pos)).map(|i| *i)
    }

    /// Returns an referenced iterator over the vector's elements
    #[inline]
    pub fn iter<'a>(&'a self) -> CVecIterRef<'a> {
        CVecIterRef::new(self)
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

    /// Returns a Vec of bytes representing the Vector. This can be used to store it in a file or
    /// send it over the network
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();

        out.write_u64::<LittleEndian>(self.items as u64).unwrap();
        out.write_u32::<LittleEndian>(self.data.len() as u32)
            .unwrap();

        for (num_bits, block) in self.data.iter() {
            let block_size = *num_bits as usize * 32;
            out.write_u8(*num_bits).unwrap();
            out.write(&block[0..block_size]).unwrap();
        }

        out
    }

    /// Creates a new `CVec` from raw bytes. This can be used together with `as_bytes`.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, std::io::Error> {
        let mut reader = Cursor::new(bytes);

        let mut new = Self::new();

        new.items = reader.read_u64::<LittleEndian>()? as usize;
        let blocks = reader.read_u32::<LittleEndian>()?;

        for _ in 0..blocks {
            let num_bits = reader.read_u8()?;
            let block_size = num_bits as usize * 32;

            let mut block = vec![0u8; block_size];

            reader.read_exact(&mut block)?;
            new.data.push((num_bits, block.clone()));
        }

        Ok(new)
    }
}

impl Compress for CVec {
    /// Compresses a Vec<u32>
    ///
    /// # Panics
    /// Panics if data.len() > 256
    #[inline]
    fn compress_block(mut data: Vec<u32>, out: &mut Vec<u8>) -> u8 {
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
}

impl Decompress for CVec {
    /// Decompress a given block at `index`
    ///
    /// Returns `None` if there is no such block.
    #[inline]
    fn decompress_block(&self, index: usize, out: &mut Vec<u32>) -> Option<()> {
        let bitpacker = BitPacker8x::new();

        let (num_bits, block) = self.data.get(index)?;

        if out.len() != BitPacker8x::BLOCK_LEN {
            out.resize(BitPacker8x::BLOCK_LEN, 0);
        }

        let compressed_len = (*num_bits as usize) * BitPacker8x::BLOCK_LEN / 8;
        bitpacker.decompress(&block[..compressed_len], out, *num_bits);

        Some(())
    }
}
