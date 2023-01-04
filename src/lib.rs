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
//! use compressed_collections::Stack;
//! 
//! let mut compressed_stack = Stack::new();
//! for _ in 0..(1024 * 1024) {
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

use serde::Deserialize;
use serde::Serialize;

mod compression;
mod deque;
mod stack;

pub use deque::Deque;
pub use stack::Stack;

/// The amount of data to buffer before compressing.
///
/// # Examples
///
///
/// This compresses every 1MB
/// ```
/// use compressed_collections::Stack;
/// use compressed_collections::ChunkSize;
/// 
/// let mut compressed_stack = Stack::new_with_options(ChunkSize::SizeElements(1024 * 1024 * 1), 2);
/// for _ in 0..(1024 * 1024 * 10) {
///     compressed_stack.push(1.0);
/// }
/// ```
///
/// Whereas this compresses every 100MB (so it will never actually get round to compressing)
/// ```
/// use compressed_collections::Stack;
/// use compressed_collections::ChunkSize;
/// 
/// let mut compressed_stack = Stack::new_with_options(ChunkSize::SizeElements(1024 * 1024 * 100), 2);
/// for _ in 0..(1024 * 1024 * 10) {
///     compressed_stack.push(1.0);
/// }
/// ```
///
/// # Low stability
/// This enum is dependent on the internal implementation, so it is likely to change frequently
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

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn simple_test() {
        let mut big_vec = Vec::new();
        let mut compressed_stack =
            Stack::new_with_options(ChunkSize::SizeElements(1024 * 99), 2);
        for _ in 0..(1024 * 100) {
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
