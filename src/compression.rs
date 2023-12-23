use brotli::enc::BrotliEncoderParams;
use brotli::CompressorWriter;
use brotli::DecompressorWriter;
use serde::Deserialize;
use serde::Serialize;
use std::io::Write;

pub fn compress<T>(x: &T, compression_level: i32) -> Box<[u8]>
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
    compressed_writer.into_inner().into_boxed_slice()
}

pub fn decompress<T>(x: &[u8]) -> T
where
    T: for<'a> Deserialize<'a>,
{
    let mut decompressor_writer = DecompressorWriter::new(Vec::new(), 4096);
    decompressor_writer.write_all(x).unwrap(); // Cannot error because we're writing to a Vec
    decompressor_writer.flush().unwrap(); // Cannot error because we're writing to a Vec
    let decompressed = decompressor_writer.into_inner().unwrap(); // Cannot error because we're writing to a Vec
    postcard::from_bytes(&decompressed).unwrap() // Only errors on OOM or incorrect `serialize`/`deserialize` implementation
}
