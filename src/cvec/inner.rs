use serde::{Serialize, Deserialize};

use super::cache::{Cache, Cached};

#[derive(Serialize, Deserialize)]
pub struct CVec<T, const CHUNK_ELEMS: usize = 1024, const COMPRESSION_LEVEL: i32 = 0, C: Cache = Cached<T, CHUNK_ELEMS>> {
    pub(super) compressed_storage: Vec<Box<[u8]>>,
    pub(super) uncompressed_buffer: Vec<T>,
    #[serde(skip)]
    pub(super) cache: C,
}

impl<T: Clone, const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32, C: Cache> Clone for CVec<T, CHUNK_ELEMS, COMPRESSION_LEVEL, C> {
    fn clone(&self) -> Self {
        Self {
            compressed_storage: self.compressed_storage.clone(),
            uncompressed_buffer: self.uncompressed_buffer.clone(),
            cache: C::default(),
        }
    }
}
impl<T: PartialEq, const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32, C: Cache> PartialEq for CVec<T, CHUNK_ELEMS, COMPRESSION_LEVEL, C> {
    fn eq(&self, other: &Self) -> bool {
        self.compressed_storage.eq(&other.compressed_storage) && self.uncompressed_buffer.eq(&other.uncompressed_buffer)
    }
}
impl<T: Eq, const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32, C: Cache> Eq for CVec<T, CHUNK_ELEMS, COMPRESSION_LEVEL, C> {}

impl<T: PartialOrd, const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32, C: Cache> PartialOrd for CVec<T, CHUNK_ELEMS, COMPRESSION_LEVEL, C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.compressed_storage.partial_cmp(&other.compressed_storage) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.uncompressed_buffer.partial_cmp(&other.uncompressed_buffer)
    }
}
impl<T: Ord, const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32, C: Cache> Ord for CVec<T, CHUNK_ELEMS, COMPRESSION_LEVEL, C> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.compressed_storage.cmp(&other.compressed_storage).then(self.uncompressed_buffer.cmp(&other.uncompressed_buffer))
    }
}

impl<T: std::hash::Hash, const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32, C: Cache> std::hash::Hash for CVec<T, CHUNK_ELEMS, COMPRESSION_LEVEL, C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.compressed_storage.hash(state);
        self.uncompressed_buffer.hash(state);
    }
}

struct CompressedElem<'a, T, const CHUNK_ELEMS: usize>(&'a Vec<Box<[u8]>>, std::marker::PhantomData<T>);
impl<'a, T, const CHUNK_ELEMS: usize> std::fmt::Debug for CompressedElem<'a, T, CHUNK_ELEMS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let elems = self.0.len() * CHUNK_ELEMS;
        write!(f, "<{elems}x compressed")?;
        let compressed_bytes = self.0.iter().map(|x| x.len()).sum::<usize>();
        if std::mem::needs_drop::<T>() {
            // Cannot accurately calculate uncompressed size
            let cb_kb = compressed_bytes / 1024;
            match (cb_kb / 1024 / 1024, cb_kb / 1024, cb_kb, compressed_bytes) {
                (0, 0, kb, cb) if kb < 4 => write!(f, " {cb}B")?,
                (0, mb, kb, _) if mb < 4 => write!(f, " {kb}KB")?,
                (gb, mb, _, _) if gb < 4 => write!(f, " {mb}MB")?,
                (gb, _, _, _) => write!(f, " {gb}GB")?,
            };
        } else {
            let uncompressed_bytes = elems * std::mem::size_of::<T>();
            let compression_ratio = 100.0 * compressed_bytes as f64 / uncompressed_bytes as f64;
            write!(f, " {compression_ratio:.2}%")?;
        }
        write!(f, ">")
    }
}
impl<T: std::fmt::Debug, const CHUNK_ELEMS: usize, const COMPRESSION_LEVEL: i32, C: Cache> std::fmt::Debug for CVec<T, CHUNK_ELEMS, COMPRESSION_LEVEL, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_list();
        if !self.compressed_storage.is_empty() {
            f.entry(&CompressedElem::<T, CHUNK_ELEMS>(&self.compressed_storage, std::marker::PhantomData));
        }
        f.entries(&self.uncompressed_buffer);
        f.finish()?;
        Ok(())
    }
}
