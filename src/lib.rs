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
//! ```no_run
//! // use compressed_collections::CVec;
//!
//! // let mut compressed_stack = CVec::new();
//! // for _ in 0..(1024 * 1024 * 100) {
//! //     compressed_stack.push(1);
//! // }
//! ```
//!
//! This only allocates around 10MB (the default buffer size), whereas the equivalent vector would be around 100MB in size.
//!
//! Design goals:
//! - Provide collections with a subset of the API of the standard equivalent wherever possible for easy dropin use.
//! - Only implement the efficient operations for each datastructure
//!
//! Datastructures:
//! - [x] CVec
//! - [x] Deque
//! - [ ] Map
// #![feature(generic_const_exprs)]

mod compression;
// mod deque;
mod cvec;

// pub use deque::Deque;
pub use cvec::{CVec, CVecUncached, CVecIntoIter, CVecIntoIterUncached};

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
        let mut compressed_stack = CVec::new::<{ 1024 * 99 }, 2>();
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
