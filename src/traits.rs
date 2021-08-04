use crate::CVec;

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

impl From<&CVec> for Vec<u32> {
    #[inline]
    fn from(cvec: &CVec) -> Self {
        cvec.iter().collect()
    }
}

impl From<CVec> for Vec<u32> {
    #[inline]
    fn from(cvec: CVec) -> Self {
        cvec.into_iter().collect::<Vec<u32>>()
    }
}

impl From<&Vec<u32>> for CVec {
    #[inline]
    fn from(vec: &Vec<u32>) -> Self {
        vec.iter().copied().collect::<Self>()
    }
}

impl From<Vec<u32>> for CVec {
    #[inline]
    fn from(vec: Vec<u32>) -> Self {
        vec.into_iter().collect::<Self>()
    }
}
