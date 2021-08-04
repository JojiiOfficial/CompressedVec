use crate::CVec;

pub(crate) trait Compress {
    fn compress_block(data: Vec<u32>, out: &mut Vec<u8>) -> u8;
}

pub(crate) trait Decompress {
    fn decompress_block(&self, index: usize, out: &mut Vec<u32>) -> Option<()>;
}

impl<T: AsRef<[u32]>> PartialEq<T> for CVec {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.iter().eq(other.as_ref().iter().copied())
    }
}

impl PartialEq for CVec {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl PartialEq<CVec> for Vec<u32> {
    #[inline]
    fn eq(&self, other: &CVec) -> bool {
        other.iter().eq(self.iter().copied())
    }
}

impl PartialEq<CVec> for [u32] {
    #[inline]
    fn eq(&self, other: &CVec) -> bool {
        other.iter().eq(self.iter().copied())
    }
}

impl PartialEq<CVec> for &[u32] {
    #[inline]
    fn eq(&self, other: &CVec) -> bool {
        other.iter().eq(self.iter().copied())
    }
}
