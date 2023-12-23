mod cache;
mod iterator;
mod inner;

use either::Either;
use serde::{Deserialize, Serialize};

use self::cache::{Cache, Cached, Uncached, CacheAccess, RcCacheAccess, RcCached};
use self::inner::CVec as CVecInner;
pub use self::iterator::{CVecIntoIter, CVecIntoIterUncached};
use crate::compression::{compress, decompress};

pub type Value<A, B> = Option<Either<A, B>>;
#[allow(non_snake_case)]
const fn Compressed<A, B>(a: A) -> Value<A, B> {
    Some(Either::Left(a))
}
#[allow(non_snake_case)]
const fn Uncompressed<A, B>(b: B) -> Value<A, B> {
    Some(Either::Right(b))
}

/// A stack which automatically compresses itself over a certain size
///
/// # Examples
///
/// ```
/// // use compressed_collections::CVec;
///
/// // let mut compressed_stack = CVec::new();
/// // for _ in 0..(1024) {
/// //     compressed_stack.push(1.0);
/// // }
/// ```
///
/// # Panics
///
/// This function should not panic (except on out of memory conditions). If it does, please submit an issue.
pub type CVec<T, const CHUNK_ELEMS: usize = 1024, const COMPRESSION_LEVEL: i32 = 0> = CVecInner<T, CHUNK_ELEMS, COMPRESSION_LEVEL, Cached<T, CHUNK_ELEMS>>;

impl<T> CVec<T, 0, 0> {
    pub fn new<const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32>() -> CVec<T, CHUNK_ELEMS, COMPRESSION_LEVEL> {
        CVecInner::default()
    }
}

/// A stack which automatically compresses itself over a certain size
///
/// # Examples
///
/// ```
/// // use compressed_collections::CVec;
///
/// // let mut compressed_stack = CVec::new();
/// // for _ in 0..(1024) {
/// //     compressed_stack.push(1.0);
/// // }
/// ```
///
/// # Panics
///
/// This function should not panic (except on out of memory conditions). If it does, please submit an issue.
pub type CVecRc<T, const CHUNK_ELEMS: usize = 1024, const COMPRESSION_LEVEL: i32 = 0> = CVecInner<T, CHUNK_ELEMS, COMPRESSION_LEVEL, RcCached<T, CHUNK_ELEMS>>;

impl<T> CVecRc<T, 0, 0> {
    pub fn new<const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32>() -> CVecRc<T, CHUNK_ELEMS, COMPRESSION_LEVEL> {
        CVecInner::default()
    }
}

/// A stack which automatically compresses itself over a certain size
///
/// # Examples
///
/// ```
/// // use compressed_collections::CVec;
///
/// // let mut compressed_stack = CVec::new();
/// // for _ in 0..(1024) {
/// //     compressed_stack.push(1.0);
/// // }
/// ```
///
/// # Panics
///
/// This function should not panic (except on out of memory conditions). If it does, please submit an issue.
pub type CVecUncached<T, const CHUNK_ELEMS: usize = 1024, const COMPRESSION_LEVEL: i32 = 0> = CVecInner<T, CHUNK_ELEMS, COMPRESSION_LEVEL, Uncached>;

impl<T> CVecUncached<T, 0, 0> {
    pub fn new<const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32>() -> CVecUncached<T, CHUNK_ELEMS, COMPRESSION_LEVEL> {
        CVecUncached::default()
    }
}

impl<T, C: Cache, const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32>Default for CVecInner<T, CHUNK_ELEMS, COMPRESSION_LEVEL, C> {
    fn default() -> Self {
        let _ = Self::ASSERT_SUPPORTED_SIZE;
        let _ = Self::COMPRESSION_LEVEL_CHECK;
        Self {
            uncompressed_buffer: Default::default(),
            compressed_storage: Default::default(),
            cache: Default::default(),
        }
    }
}

impl<T, C: Cache, const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32> CVecInner<T, CHUNK_ELEMS, COMPRESSION_LEVEL, C> {
    const ASSERT_SUPPORTED_SIZE: () = assert!(CHUNK_ELEMS > 0, "Chunk size must be greater than 0");
    const COMPRESSION_LEVEL_CHECK: () = assert!(COMPRESSION_LEVEL >= 0 && COMPRESSION_LEVEL <= 11, "Compression level must be between 0 and 11");

    pub fn push(&mut self, value: T)
    where
        T: Serialize,
    {
        self.uncompressed_buffer.push(value);
        if self.uncompressed_buffer.len() >= CHUNK_ELEMS {
            let compressed = compress(&self.uncompressed_buffer, COMPRESSION_LEVEL);
            self.compressed_storage.push(compressed);
            self.uncompressed_buffer.clear();
        }
    }
    pub fn pop(&mut self) -> Option<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        if self.uncompressed_buffer.is_empty() {
            if let Some(x) = self.compressed_storage.pop() {
                self.uncompressed_buffer = decompress(&x);
                if self.cache.is_cached(self.compressed_storage.len()) {
                    self.cache.kill_all();
                }
            }
        }
        self.uncompressed_buffer.pop()
    }
    pub fn len(&self) -> usize {
        self.uncompressed_buffer.len() + self.compressed_storage.len() * CHUNK_ELEMS
    }
    pub fn is_empty(&self) -> bool {
        self.uncompressed_buffer.is_empty() && self.compressed_storage.is_empty()
    }
    #[must_use]
    pub fn get_uncached(&self, idx: usize) -> Value<T, &T> where T: for<'a> Deserialize<'a> {
        match self.split(idx)? {
            Either::Left((chunk_idx, chunk_offset)) => {
                let data: Vec<T> = decompress(&self.compressed_storage[chunk_idx]);
                Compressed(data.into_iter().nth(chunk_offset).unwrap())
            }
            Either::Right(elem) =>
                Uncompressed(&self.uncompressed_buffer[elem]),
        }
    }
    #[must_use]
    pub fn get_ref(&mut self, idx: usize) -> Option<&T> where T: for<'a> Deserialize<'a>, C: CacheAccess<T> {
        match self.split(idx)? {
            Either::Left((chunk_idx, chunk_offset)) => {
                let data = &self.compressed_storage[chunk_idx];
                Some(self.cache.get_compressed(chunk_idx, chunk_offset, data))
            }
            Either::Right(elem) =>
                Some(&self.uncompressed_buffer[elem]),
        }
    }

    #[must_use]
    pub fn get(&self, idx: usize) -> Option<T> where T: for<'a> Deserialize<'a> + Clone, C: RcCacheAccess<T, CHUNK_ELEMS> {
        match self.split(idx)? {
            Either::Left((chunk_idx, chunk_offset)) => {
                let data = &self.compressed_storage[chunk_idx];
                Some(self.cache.get_compressed(chunk_idx, chunk_offset, data).borrow().clone())
            }
            Either::Right(elem) =>
                Some(self.uncompressed_buffer[elem].clone()),
        }
    }
    // #[must_use]
    // pub fn get_rc(&self, idx: usize) -> Option<Entry<'_, T, CHUNK_ELEMS>> where T: for<'a> Deserialize<'a>, C: RcCacheAccess<T, CHUNK_ELEMS> {
    //     match self.split(idx)? {
    //         Either::Left((chunk_idx, chunk_offset)) => {
    //             let data = &self.compressed_storage[chunk_idx];
    //             Some(self.cache.get_compressed(chunk_idx, chunk_offset, data))
    //         }
    //         Either::Right(elem) =>
    //             Some(Entry::Uncompressed(&self.uncompressed_buffer[elem])),
    //     }
    // }
    // #[must_use]
    // pub fn get<'c>(&'c mut self, idx: usize) -> Option<C::Item> where C: CacheAccess<'c, T> {
    //     match self.split(idx) {
    //         Ok(idx) => self.uncompressed_buffer.get(idx).map(C::get_uncompressed),
    //         Err((chunk_idx, chunk_offset)) => {
    //             let data = &self.compressed_storage[chunk_idx];
    //             Some(self.cache.get_compressed(chunk_idx, chunk_offset, data))
    //         }
    //     }
    // }

    fn split(&self, idx: usize) -> Value<(usize, usize), usize> {
        let (idx, offset) = (idx / CHUNK_ELEMS, idx % CHUNK_ELEMS);
        if idx < self.compressed_storage.len() {
            Compressed((idx, offset))
        } else if idx == self.compressed_storage.len() && offset < self.uncompressed_buffer.len() {
            Uncompressed(offset)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{*, compression::{compress, decompress}};

    #[test]
    fn simple_test() {
        let mut big_vec = Vec::new();
        let mut compressed_stack = CVec::new::<{ 1024 * 9 }, 0>();
        // Test `push`
        for i in 0..(1024 * 10) {
            big_vec.push(i as f64);
            compressed_stack.push(i as f64);
        }
        // Test `get`
        for idx in 0..(1024 * 10) + 1 {
            let bv = big_vec.get(idx);
            let cv = compressed_stack.get_ref(idx);
            assert_eq!(bv, cv);
        }
        // Test `compress` and `decompress`
        let data = compress(&compressed_stack, 0);
        compressed_stack = decompress(&data);
        for idx in 0..(1024 * 10) + 1 {
            let bv = big_vec.get(idx);
            let cv = compressed_stack.get_ref(idx);
            assert_eq!(bv, cv);
        }
        // Test `pop`
        loop {
            let a = big_vec.pop();
            let b = compressed_stack.pop();
            assert_eq!(a, b);
            if a.is_none() | b.is_none() {
                break;
            }
        }
    }

    #[test]
    fn iter_test() {
        let mut big_vec = Vec::new();
        let mut compressed_stack = CVec::new::<{ 1024 * 9 }, 0>();
        for _ in 0..(1024 * 10) {
            big_vec.push(1.0);
            compressed_stack.push(1.0);
        }
        for (a, b) in big_vec.iter().zip(&compressed_stack) {
            assert_eq!(*a, b);
        }
        let mut big_vec_it = big_vec.into_iter();
        let mut compressed_stack_it = compressed_stack.into_iter();
        loop {
            let a = big_vec_it.next();
            let b = compressed_stack_it.next();
            assert_eq!(a, b);
            if a.is_none() | b.is_none() {
                break;
            }
        }
    }

    #[test]
    fn rec_test() {
        let mut compressed_vec = CVec::new::<{ 8 * 10 - 1 }, 11>();
        for j in 0..(8 * 10) {
            let mut item = CVec::new::<{ 128 * 10 - 1 }, 0>();
            for i in j..j + (128 * 10) {
                item.push(i as f64);
            }
            compressed_vec.push(item);
        }
        println!("CV: {:?}", compressed_vec);
    }
}
