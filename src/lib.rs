use std::io::Write;

use brotli::enc::BrotliEncoderParams;
use brotli::CompressorWriter;
use brotli::DecompressorWriter;
use serde::*;

pub enum ChunkSize {
    SizeElements(usize),
    SizeBytes(usize),
    SizeMB(usize),
    Auto,
}

pub struct CompressedStack<T> {
    uncompressed_buffer: Vec<T>,
    compressed_storage: Vec<Vec<u8>>,
    chunk_size: usize,
}

impl<T> CompressedStack<T> {
    pub fn new() -> CompressedStack<T> {
        CompressedStack::new_with_options(ChunkSize::Auto)
    }

    pub fn new_with_options(chunksize: ChunkSize) -> CompressedStack<T> {
        let elementsize = std::mem::size_of::<T>();
        let chunk_size = match chunksize {
            ChunkSize::SizeElements(x) => x,
            ChunkSize::SizeBytes(x) => x / elementsize,
            ChunkSize::SizeMB(x) => x * 1024 * 1024 / elementsize,
            ChunkSize::Auto => 10 * 1024 * 1024 / elementsize,
        };
        let uncompressed_buffer = Vec::new();
        let compressed_storage = Vec::new();
        CompressedStack {
            uncompressed_buffer,
            compressed_storage,
            chunk_size,
        }
    }
    pub fn push(&mut self, value: T)
    where
        T: Serialize,
    {
        self.uncompressed_buffer.push(value);
        if self.uncompressed_buffer.len() >= self.chunk_size {
            let compressed = compress(&self.uncompressed_buffer);
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
            }
        }
        self.uncompressed_buffer.pop()
    }
}

impl<T> Default for CompressedStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

fn compress<T>(x: &Vec<T>) -> Vec<u8>
where
    T: Serialize,
{
    let serialized = postcard::to_stdvec(x).unwrap();
    let params = BrotliEncoderParams {
        quality: 6,
        ..Default::default()
    };
    let mut compressed_writer = CompressorWriter::with_params(Vec::new(), 4096, &params);
    compressed_writer.write_all(&serialized).unwrap();
    compressed_writer.flush().unwrap();
    compressed_writer.into_inner()
}

fn decompress<T>(x: &[u8]) -> Vec<T>
where
    T: for<'a> Deserialize<'a>,
{
    let mut decompressor_writer = DecompressorWriter::new(Vec::new(), 4096);
    decompressor_writer.write_all(x).unwrap();
    decompressor_writer.flush().unwrap();
    let decompressed = decompressor_writer.into_inner().unwrap();
    postcard::from_bytes(&decompressed).unwrap()
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
            CompressedStack::new_with_options(ChunkSize::SizeElements(1024 * 1024 * 99));
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
