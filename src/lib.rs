use std::{
    io::{Cursor, Read, Write},
    mem::size_of,
};

use bitpacking::{BitPacker, BitPacker8x};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

/// A compressed Vec<u32>
#[derive(Clone, Debug, PartialEq, Eq)]
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

    /// Allocate a new Compressed vector which can store `capacity` numbers without reallocating
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let req_blocks = Self::req_block_count(capacity);
        let mut data = Vec::with_capacity(req_blocks);

        for _ in 0..req_blocks {
            data.push((0, Vec::with_capacity(1024)));
        }

        Self { data, items: 0 }
    }

    /// Returns the amount of allocated bytes by the vector
    #[inline]
    pub fn byte_len(&self) -> usize {
        // `items` and initial `data` vec
        let mut len = size_of::<usize>() * 2;

        for block in self.data.iter() {
            // u8
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
            let (enc, num_bits) = Self::compress_block(vec![val]);
            self.data.push((num_bits, enc));
        } else {
            let block_nr = self.last_block();

            // decompress last block
            let mut block = self.decompress_block(block_nr).unwrap();

            // Set value at position
            block[self.items % 256] = val;

            // Compress block again
            let (enc, num_bits) = Self::compress_block(block);
            *self.data.get_mut(block_nr).unwrap() = (num_bits, enc);
        }

        self.items += 1;
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
            let block_nr = self.items / 256;
            self.data.remove(block_nr);
        }

        Some(popped)
    }

    /// Returns the last item of the vec
    #[inline]
    pub fn last(&self) -> Option<u32> {
        self.get(self.len() - 1)
    }

    /// Returns the u32 at `pos`
    #[inline]
    pub fn get(&self, pos: usize) -> Option<u32> {
        if pos >= self.items {
            return None;
        }
        self.decompress_block(pos / 256)?.get(pos % 256).map(|i| *i)
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

    /// Returns the index in `self.data` of the last block
    fn last_block(&self) -> usize {
        self.items / 256
    }

    /// Returns the amount of blocks required to store `size` elements
    fn req_block_count(size: usize) -> usize {
        if size % 256 != 0 {
            (size / 256) + 1
        } else {
            size / 256
        }
    }

    /// Compresse a Vec<u32>
    ///
    /// # Panics
    /// Panics if data.len() > 256
    fn compress_block(mut data: Vec<u32>) -> (Vec<u8>, u8) {
        assert!(data.len() <= 256);

        if data.len() < 256 {
            data.extend((0..(256 - data.len() as u32 % 256)).map(|_| 0));
        }

        let bitpacker = BitPacker8x::new();
        let num_bits: u8 = bitpacker.num_bits(&data);
        let mut compressed = vec![0u8; 4 * BitPacker8x::BLOCK_LEN];
        let compressed_len = bitpacker.compress(&data, &mut compressed[..], num_bits);
        (compressed[..compressed_len].to_owned(), num_bits)
    }

    /// Decompress a given block
    ///
    /// Returns `None` if there is no such block.
    fn decompress_block(&self, index: usize) -> Option<Vec<u32>> {
        let bitpacker = BitPacker8x::new();
        let (num_bits, block) = self.data.get(index)?;
        let mut decompressed = vec![0u32; BitPacker8x::BLOCK_LEN];

        let compressed_len = (*num_bits as usize) * BitPacker8x::BLOCK_LEN / 8;

        bitpacker.decompress(&block[..compressed_len], &mut decompressed[..], *num_bits);

        Some(decompressed)
    }

    /// Returns true if a new block needs to be allocated.
    #[inline]
    fn need_new_block(&self) -> bool {
        self.items / 256 >= self.data.len()
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
    fn test_push_with_capacity() {
        let test_data = (0..9000).collect::<Vec<_>>();

        let mut v = CVec::with_capacity(10000);
        for i in test_data.iter() {
            v.push(*i);
        }
        assert_eq!(v.len(), test_data.len());

        for (pos, i) in test_data.iter().enumerate() {
            assert_eq!(v.get(pos).unwrap(), *i);
        }
    }

    #[test]
    fn test_pop_with_capacity() {
        let mut v = CVec::with_capacity(1000);
        let mut rv = Vec::new();
        let test_data = (0..20).collect::<Vec<_>>();

        for (pos, i) in test_data.iter().enumerate() {
            v.push(*i);
            rv.push(*i);

            if pos % 2 == 0 {
                v.pop();
                rv.pop();
            }
        }

        let new_len = test_data.len() / 2;

        assert!(v.len() == new_len);
        assert!(rv.len() == v.len());

        for _ in 0..new_len {
            assert_eq!(v.pop(), rv.pop());
        }

        let test_data = (0..4999).collect::<Vec<_>>();

        let mut v = CVec::new();
        for i in test_data.iter() {
            v.push(*i);
        }

        for i in test_data.iter().rev() {
            assert_eq!(v.pop().unwrap(), *i);
        }
    }

    #[test]
    fn test_push() {
        let test_data = (0..4999).collect::<Vec<_>>();

        let mut v = CVec::new();
        for i in test_data.iter() {
            v.push(*i);
        }
        assert_eq!(v.len(), test_data.len());

        for (pos, i) in test_data.iter().enumerate() {
            assert_eq!(v.get(pos).unwrap(), *i);
        }
    }

    #[test]
    fn test_pop_simple() {
        let mut v = CVec::new();
        let test_data = (0..1024).collect::<Vec<_>>();
        for i in test_data.iter() {
            v.push(*i);
        }

        for i in test_data.iter().rev() {
            assert_eq!(v.pop().unwrap(), *i);
        }

        assert!(v.data.is_empty());
    }

    #[test]
    fn test_pop() {
        let mut v = CVec::new();
        let mut rv = Vec::new();
        let test_data = (0..20).collect::<Vec<_>>();

        for (pos, i) in test_data.iter().enumerate() {
            v.push(*i);
            rv.push(*i);

            if pos % 2 == 0 {
                v.pop();
                rv.pop();
            }
        }

        let new_len = test_data.len() / 2;

        assert!(v.len() == new_len);
        assert!(rv.len() == v.len());

        for _ in 0..new_len {
            assert_eq!(v.pop(), rv.pop());
        }

        assert!(rv.is_empty());
        assert!(v.data.is_empty());
        assert!(v.is_empty());
    }

    #[test]
    fn test_capacity() {
        let v = CVec::with_capacity(1000);
        assert_eq!(v.capacity(), 1024);
    }

    #[test]
    fn test_encoding() {
        let mut v = CVec::new();
        let test_data = (0..9999).collect::<Vec<_>>();
        for i in test_data.iter() {
            v.push(*i);
        }

        let bytes = v.as_bytes();
        println!("len: {}", bytes.len());
        println!("raw len: {}", test_data.len() * 4);

        let new = CVec::from_bytes(&bytes);
        assert!(new.is_ok());
        assert_eq!(new.unwrap(), v);
    }

    #[test]
    fn test_req_blocks() {
        assert_eq!(CVec::req_block_count(0), 0);
        assert_eq!(CVec::req_block_count(1), 1);
        assert_eq!(CVec::req_block_count(256), 1);
        assert_eq!(CVec::req_block_count(257), 2);
        assert_eq!(CVec::req_block_count(512), 2);
        assert_eq!(CVec::req_block_count(513), 3);
        assert_eq!(CVec::req_block_count(1024), 4);
    }
}
