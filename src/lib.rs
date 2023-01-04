#![warn(missing_docs)]

//! Collections which transparently compress data to reduce memory usage
//!
//! This crate offers collections which automatically compress themselves, which reduces memory usage, sometimes
//! substantially, allowing collections to be held in memory that would otherwise be too big.
//!
//! So far, a stack is implemented, which can be used as a dropin replacement for a Vec stack.
//! The only restriction on the datatypes in the collections is that they must be serde serializable.
//!
//! For instance:
//!
//! ```
//! let mut compressed_stack = Stack::new();
//! for _ in 0..(1024 * 1024 * 1024) {
//!     compressed_stack.push(1.0);
//! }
//! ```
//!
//! This only allocates around 10MB (the default buffer size), whereas the equivalent vector would be around 4GB in size.
//!
//! Design goals:
//! - Provide collections with a subset of the API of the standard equivalent wherever possible for easy dropin use.
//! - Only implement the efficient operations for each datastructure
//!
//! Datastructures:
//! - [x] Stack
//! - [ ] Deque
//! - [ ] Hashmap

use brotli::enc::BrotliEncoderParams;
use brotli::CompressorWriter;
use brotli::DecompressorWriter;
use serde::*;
use std::io::Write;

/// The amount of data to buffer before compressing.
/// /// # Examples
///
///
/// This compresses every 1MB
/// ```
/// let mut compressed_stack = Stack::new_with_options(ChunkSize::SizeElements(1024 * 1024 * 1), 2);
/// for _ in 0..(1024 * 1024 * 10) {
///     compressed_stack.push(1.0);
/// }
/// ```
///
/// Whereas this compresses every 100MB (so it will never actually get round to compressing)
/// ```
/// let mut compressed_stack = Stack::new_with_options(ChunkSize::SizeElements(1024 * 1024 * 100), 2);
/// for _ in 0..(1024 * 1024 * 10) {
///     compressed_stack.push(1.0);
/// }
/// ```
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Serialize, Deserialize,
)]
pub enum ChunkSize {
    /// Maximum buffer length
    SizeElements(usize),
    /// Maximum buffer size in bytes
    SizeBytes(usize),
    /// Maximum buffer size in MB
    SizeMB(usize),
    /// Default value (10MB, subject to change)
    #[default]
    Default,
}

/// A stack which automatically compresses itself over a certain size
///
/// # Examples
///
/// ```
/// let mut compressed_stack = Stack::new();
/// for _ in 0..(1024 * 1024 * 1024) {
///     compressed_stack.push(1.0);
/// }
///
/// # Panics
///
/// This function should not panic (except potentially on out of memory conditions). If it does, please submit an issue.
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Stack<T> {
    uncompressed_buffer: Vec<T>,
    compressed_storage: Vec<Vec<u8>>,
    chunk_size: usize,
    compression_level: i32,
}

impl<T> Stack<T> {
    /// Constructor with default options
    pub fn new() -> Stack<T> {
        Stack::new_with_options(ChunkSize::Default, 6)
    }

    /// Constructor with customisable options
    ///
    /// - `chunksize` - size of chunks to compress, see [`ChunkSize`]
    /// - `compression_level` - Brotli compression level (0-9) default is 6
    pub fn new_with_options(chunksize: ChunkSize, compression_level: i32) -> Stack<T> {
        let elementsize = std::mem::size_of::<T>();
        let chunk_size = match chunksize {
            ChunkSize::SizeElements(x) => x,
            ChunkSize::SizeBytes(x) => x / elementsize,
            ChunkSize::SizeMB(x) => x * 1024 * 1024 / elementsize,
            ChunkSize::Default => 10 * 1024 * 1024 / elementsize,
        };
        let uncompressed_buffer = Vec::new();
        let compressed_storage = Vec::new();
        Stack {
            uncompressed_buffer,
            compressed_storage,
            chunk_size,
            compression_level,
        }
    }
    /// Push an item onto the stack
    pub fn push(&mut self, value: T)
    where
        T: Serialize,
    {
        self.uncompressed_buffer.push(value);
        if self.uncompressed_buffer.len() >= self.chunk_size {
            let compressed = compress(&self.uncompressed_buffer, self.compression_level);
            self.compressed_storage.push(compressed);
            self.uncompressed_buffer.clear();
        }
    }
    /// Pop an item off the stack
    pub fn pop(&mut self) -> Option<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        if self.uncompressed_buffer.is_empty() {
            if let Some(x) = self.compressed_storage.pop() {
                self.uncompressed_buffer = decompress(&x);
            }
        }
        self.uncompressed_buffer.pop()
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self::new()
    }
}

fn compress<T>(x: &Vec<T>, compression_level: i32) -> Vec<u8>
where
    T: Serialize,
{
    let serialized = postcard::to_stdvec(x).unwrap(); // Only errors on OOM
    let params = BrotliEncoderParams {
        quality: compression_level,
        ..Default::default()
    };
    let mut compressed_writer = CompressorWriter::with_params(Vec::new(), 4096, &params);
    compressed_writer.write_all(&serialized).unwrap(); // Cannot error because we're writing to a Vec
    compressed_writer.flush().unwrap(); // Cannot error because we're writing to a Vec
    compressed_writer.into_inner()
}

fn decompress<T>(x: &[u8]) -> Vec<T>
where
    T: for<'a> Deserialize<'a>,
{
    let mut decompressor_writer = DecompressorWriter::new(Vec::new(), 4096);
    decompressor_writer.write_all(x).unwrap(); // Cannot error because we're writing to a Vec
    decompressor_writer.flush().unwrap(); // Cannot error because we're writing to a Vec
    let decompressed = decompressor_writer.into_inner().unwrap(); // Cannot error because we're writing to a Vec
    postcard::from_bytes(&decompressed).unwrap() // Only errors on OOM
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn simple_test() {
        let mut big_vec = Vec::new();
        let mut compressed_stack =
            Stack::new_with_options(ChunkSize::SizeElements(1024 * 1024 * 99), 2);
        for _ in 0..(1024 * 1024 * 100) {
            big_vec.push(1.0);
            compressed_stack.push(1.0);
        }
        loop {
            let a = big_vec.pop();
            let b = compressed_stack.pop();
            assert!(a == b);
            if a.is_none() | b.is_none() {
                break;
            }
        }
    }
}
