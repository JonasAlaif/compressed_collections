use std::collections::VecDeque;

use serde::Deserialize;
use serde::Serialize;

use crate::compression::compress;
use crate::compression::decompress;
use crate::ChunkSize;

/// A deque which automatically compresses itself over a certain size
///
/// # Examples
///
/// ```
/// use compressed_collections::Deque;
///
/// let mut compressed_deque = Deque::new();
/// for _ in 0..(1024) {
///     compressed_deque.push_back(1);
/// }
/// while let Some(x) = compressed_deque.pop_front(){
///     assert!(x==1);
/// }
/// ```
///
/// # Panics
///
/// This function should not panic (except on out of memory conditions). If it does, please submit an issue.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Deque<T> {
    uncompressed_buffer_front: VecDeque<T>,
    uncompressed_buffer_back: VecDeque<T>,
    compressed_storage: VecDeque<Box<[u8]>>,
    chunk_size: usize,
    compression_level: i32,
    length: usize,
}

impl<T> Deque<T> {
    /// Constructor with default options
    pub fn new() -> Deque<T> {
        Deque::new_with_options(ChunkSize::Default, 0)
    }
    /// Constructor with customisable options
    ///
    /// - `chunksize` size of chunks to compress, see [`ChunkSize`]
    /// - `compression_level` Brotli compression level (0-9) default is 0
    ///
    /// # Low stability
    /// This constructor is dependent on the internal implementation, so it is likely to change more frequently than [`Deque::new`]
    pub fn new_with_options(chunksize: ChunkSize, compression_level: i32) -> Deque<T> {
        let elementsize = std::mem::size_of::<T>();
        let chunk_size = match chunksize {
            ChunkSize::SizeElements(x) => x,
            ChunkSize::SizeBytes(x) => x / elementsize,
            ChunkSize::SizeMB(x) => x * 1024 * 1024 / elementsize,
            ChunkSize::Default => 10 * 1024 * 1024 / elementsize,
        };
        let uncompressed_buffer_front = VecDeque::new();
        let uncompressed_buffer_back = VecDeque::new();
        let compressed_storage = VecDeque::new();
        let length = 0;
        Deque {
            uncompressed_buffer_front,
            uncompressed_buffer_back,
            compressed_storage,
            chunk_size,
            compression_level,
            length,
        }
    }
    /// Appends an element to the back of the deque.
    pub fn push_back(&mut self, value: T)
    where
        T: Serialize,
    {
        self.uncompressed_buffer_back.push_back(value);
        self.length += 1;
        if self.uncompressed_buffer_back.len() >= self.chunk_size {
            let compressed = compress(&self.uncompressed_buffer_back, self.compression_level);
            self.compressed_storage.push_back(compressed);
            self.uncompressed_buffer_back.clear();
        }
    }
    /// Appends an element to the front of the deque.
    pub fn push_front(&mut self, value: T)
    where
        T: Serialize,
    {
        self.uncompressed_buffer_front.push_front(value);
        self.length += 1;
        if self.uncompressed_buffer_front.len() >= self.chunk_size {
            let compressed = compress(&self.uncompressed_buffer_front, self.compression_level);
            self.compressed_storage.push_front(compressed);
            self.uncompressed_buffer_front.clear();
        }
    }
    /// Removes the last element from the deque and returns it, or None if it is empty.
    pub fn pop_back(&mut self) -> Option<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        if self.uncompressed_buffer_back.is_empty() {
            if let Some(x) = self.compressed_storage.pop_back() {
                self.uncompressed_buffer_back = decompress(&x);
            } else {
                self.uncompressed_buffer_back = std::mem::take(&mut self.uncompressed_buffer_front);
            }
        }
        let result = self.uncompressed_buffer_back.pop_back();
        if result.is_some() {
            self.length -= 1;
        }
        result
    }
    /// Removes the first element from the deque and returns it, or None if it is empty.
    pub fn pop_front(&mut self) -> Option<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        if self.uncompressed_buffer_front.is_empty() {
            if let Some(x) = self.compressed_storage.pop_front() {
                self.uncompressed_buffer_front = decompress(&x);
            } else {
                self.uncompressed_buffer_front = std::mem::take(&mut self.uncompressed_buffer_back);
            }
        }
        let result = self.uncompressed_buffer_front.pop_front();
        if result.is_some() {
            self.length -= 1;
        }
        result
    }
    /// Returns the number of elements in the deque, also referred to as its ‘length’.
    pub fn len(&self) -> usize {
        self.length
    }
    /// Returns true if the deque has a length of 0.
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

impl<T> Default for Deque<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Iterator for Deque<T>
where
    T: Serialize + for<'a> Deserialize<'a>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        self.pop_front()
    }
}

impl<T> FromIterator<T> for Deque<T>
where
    T: Serialize + for<'a> Deserialize<'a>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        let mut c = Deque::new();
        for i in iter {
            c.push_back(i);
        }
        c
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn simple_test() {
        let mut big_vecdeque = std::collections::VecDeque::new();
        let mut compressed_deque = Deque::new_with_options(ChunkSize::SizeElements(1024 * 9), 0);
        for _ in 0..(1024 * 10) {
            big_vecdeque.push_back(1);
            compressed_deque.push_back(1);
        }
        loop {
            let a = big_vecdeque.pop_front();
            let b = compressed_deque.pop_front();
            assert!(a == b);
            if a.is_none() | b.is_none() {
                break;
            }
        }
    }

    #[test]
    fn iter_test() {
        let mut big_vecdeque = Vec::new();
        let mut compressed_deque = Stack::new_with_chunk_elems::<{ 1024 * 9 }, 0, 1>();
        for _ in 0..(1024 * 10) {
            big_vecdeque.push(1.0);
            compressed_deque.push(1.0);
        }
        let mut big_vecdeque_it = big_vecdeque.into_iter();
        let mut compressed_deque_it = compressed_deque;
        loop {
            let a = big_vecdeque_it.next();
            let b = compressed_deque_it.next();
            assert!(a == b);
            if a.is_none() | b.is_none() {
                break;
            }
        }
    }
}
