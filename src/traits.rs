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

impl<T: Into<u32> + Copy> From<&Vec<T>> for CVec {
    #[inline]
    fn from(vec: &Vec<T>) -> Self {
        vec.iter().map(|i| (*i).into()).collect::<CVec>()
    }
}

impl<T: Into<u32>> From<Vec<T>> for CVec {
    #[inline]
    fn from(vec: Vec<T>) -> Self {
        vec.into_iter().map(|i| i.into()).collect::<CVec>()
    }
}

impl<T: From<u32>> From<&CVec> for Vec<T> {
    #[inline]
    fn from(cvec: &CVec) -> Self {
        cvec.iter().map(|i| i.into()).collect()
    }
}

impl<T: From<u32>> From<CVec> for Vec<T> {
    #[inline]
    fn from(cvec: CVec) -> Self {
        cvec.into_iter().map(|i| T::from(i)).collect::<Vec<T>>()
    }
}
