use either::Either;
use serde::{Deserialize, Serialize};

use crate::compression::decompress;

use super::{inner::CVec as CVecInner, cache::{Cache, Cached, Uncached}};

// Owned Iterator

/// TODO: doc
pub type CVecIntoIter<T, const CHUNK_ELEMS: usize = 1024, const COMPRESSION_LEVEL: i32 = 0> = CVecIntoIterInner<T, CHUNK_ELEMS, COMPRESSION_LEVEL, Cached<T, CHUNK_ELEMS>>;
/// TODO: doc
pub type CVecIntoIterUncached<T, const CHUNK_ELEMS: usize = 1024, const COMPRESSION_LEVEL: i32 = 0> = CVecIntoIterInner<T, CHUNK_ELEMS, COMPRESSION_LEVEL, Uncached>;

use inner::CVecIntoIter as CVecIntoIterInner;
mod inner {
    use super::*;
    pub struct CVecIntoIter<T, const CHUNK_ELEMS: usize = 1024, const COMPRESSION_LEVEL: i32 = 0, C: Cache = Cached<T, CHUNK_ELEMS>> {
        pub(super) inner: CVecInner<T, CHUNK_ELEMS, COMPRESSION_LEVEL, C>,
    }
}

impl<T, C: Cache, const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32> Iterator for CVecIntoIterInner<T, CHUNK_ELEMS, COMPRESSION_LEVEL, C>
where
    T: for<'a> Deserialize<'a>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.inner.len(), Some(self.inner.len()))
    }
}

// Owned IntoIterator

impl<T, C: Cache, const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32> IntoIterator for CVecInner<T, CHUNK_ELEMS, COMPRESSION_LEVEL, C>
where
    T: for<'a> Deserialize<'a>,
{
    type Item = T;
    type IntoIter = CVecIntoIterInner<T, CHUNK_ELEMS, COMPRESSION_LEVEL, C>;

    fn into_iter(self) -> Self::IntoIter {
        CVecIntoIterInner { inner: self }
    }
}

// // Shared borrow RC Iterator

// pub struct CVecIterRc<'i, T, const BORROW: bool = false, const CHUNK_ELEMS: usize = 1024, const COMPRESSION_LEVEL: i32 = 0, C: Cache + RcCacheAccess<T, CHUNK_ELEMS> = RcCached<T, CHUNK_ELEMS>> {
//     idx: usize,
//     inner: &'i CVecInner<T, CHUNK_ELEMS, COMPRESSION_LEVEL, C>,
// }

// impl<'i, T, C: Cache + RcCacheAccess<T, CHUNK_ELEMS>, const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32> CVecIterRc<'i, T, false, CHUNK_ELEMS, COMPRESSION_LEVEL, C> {
//     pub fn borrow(&self) -> CVecIterRc<'i, T, true, CHUNK_ELEMS, COMPRESSION_LEVEL, C> {
//         CVecIterRc { idx: self.idx, inner: self.inner }
//     }
// }

// impl<'i, T, C: Cache, const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32> Iterator for CVecIterRc<'i, T, false, CHUNK_ELEMS, COMPRESSION_LEVEL, C>
// where
//     T: for<'a> Deserialize<'a>,
//     C: RcCacheAccess<T, CHUNK_ELEMS>,
// {
//     type Item = Entry<'i, T, CHUNK_ELEMS>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let idx = self.idx;
//         self.idx += 1;
//         self.inner.get_rc(idx)
//     }
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         (self.inner.len().saturating_sub(self.idx), Some(self.inner.len().saturating_sub(self.idx)))
//     }
// }

// impl<'i, T, C: Cache, const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32> Iterator for CVecIterRc<'i, T, true, CHUNK_ELEMS, COMPRESSION_LEVEL, C>
// where
//     T: for<'a> Deserialize<'a>,
//     C: RcCacheAccess<T, CHUNK_ELEMS>,
// {
//     type Item = EntryRef<'i, T, CHUNK_ELEMS>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let idx = self.idx;
//         self.idx += 1;
//         Some(self.inner.get_rc(idx)?.borrow())
//     }
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         (self.inner.len().saturating_sub(self.idx), Some(self.inner.len().saturating_sub(self.idx)))
//     }
// }

// // Shared borrow RC IntoIterator

// impl<'i, T, C: Cache, const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32> IntoIterator for &'i CVecInner<T, CHUNK_ELEMS, COMPRESSION_LEVEL, C>
// where
//     T: for<'a> Deserialize<'a>,
//     C: RcCacheAccess<T, CHUNK_ELEMS>,
// {
//     type Item = EntryRef<'i, T, CHUNK_ELEMS>;
//     type IntoIter = CVecIterRc<'i, T, true, CHUNK_ELEMS, COMPRESSION_LEVEL, C>;

//     fn into_iter(self) -> Self::IntoIter {
//         CVecIterRc { idx: 0, inner: self }
//     }
// }

// Shared borrow Iterator

pub struct CVecIter<'i, T, const CHUNK_ELEMS: usize = 1024, const COMPRESSION_LEVEL: i32 = 0, C: Cache = Uncached> {
    chunk_idx: usize,
    inner: &'i CVecInner<T, CHUNK_ELEMS, COMPRESSION_LEVEL, C>,
    iter: Either<std::vec::IntoIter<T>, std::slice::Iter<'i, T>>,
}

impl<'i, T, C: Cache, const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32> Iterator for CVecIter<'i, T, CHUNK_ELEMS, COMPRESSION_LEVEL, C>
where
    T: Clone + for<'a> Deserialize<'a>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iter.as_ref().either(|i| i.len(), |i| i.len()) == 0 {
            if let Some(x) = self.inner.compressed_storage.get(self.chunk_idx) {
                let data: Vec<T> = decompress(x);
                self.iter = Either::Left(data.into_iter());
            } else if self.chunk_idx == self.inner.compressed_storage.len() {
                self.iter = Either::Right(self.inner.uncompressed_buffer.iter());
            } else {
                return None;
            }
            self.chunk_idx += 1;
        }
        self.iter.as_mut().either(|i| i.next(), |i| i.next().cloned())
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.iter.as_ref().either(|i| i.len(), |i| i.len());
        let compressed_len = self.inner.compressed_storage.len().saturating_sub(self.chunk_idx) * CHUNK_ELEMS;
        let uncompressed_len = if self.chunk_idx <= self.inner.compressed_storage.len() {
            self.inner.uncompressed_buffer.len()
        } else {
            0
        };
        (len + compressed_len + uncompressed_len, Some(len + compressed_len + uncompressed_len))
    }
}
impl<'i, T, C: Cache, const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32> ExactSizeIterator for CVecIter<'i, T, CHUNK_ELEMS, COMPRESSION_LEVEL, C>
where
    T: Clone + for<'a> Deserialize<'a>,
{}

// Shared borrow IntoIterator

impl<'i, T, C: Cache, const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32> IntoIterator for &'i CVecInner<T, CHUNK_ELEMS, COMPRESSION_LEVEL, C>
where
    T: Clone + for<'a> Deserialize<'a>,
{
    type Item = T;
    type IntoIter = CVecIter<'i, T, CHUNK_ELEMS, COMPRESSION_LEVEL, C>;

    fn into_iter(self) -> Self::IntoIter {
        CVecIter { chunk_idx: 0, inner: self, iter: Either::Right([].iter()) }
    }
}

// FromIterator

impl<T, C: Cache, const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32> FromIterator<T> for CVecInner<T, CHUNK_ELEMS, COMPRESSION_LEVEL, C>
where
    T: Serialize
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut c = Self::default();
        for i in iter {
            c.push(i);
        }
        c
    }
}
