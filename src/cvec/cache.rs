use std::cell::{RefCell, Ref};

use serde::{Deserialize, de::Error};

use crate::compression::decompress;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub(super) struct CacheLine<T, const CHUNK_ELEMS: usize>(Box<[T; CHUNK_ELEMS]>);

#[derive(Default)]
pub struct Uncached;

pub struct Cached<T, const CHUNK_ELEMS: usize> {
    index: usize,
    data: Option<CacheLine<T, CHUNK_ELEMS>>,
}

impl<T, const CHUNK_ELEMS: usize> Default for Cached<T, CHUNK_ELEMS> {
    fn default() -> Self {
        Self { index: usize::MAX, data: None }
    }
}

impl<T, const CHUNK_ELEMS: usize> Cached<T, CHUNK_ELEMS> {
    pub fn fill_cache(&mut self, index: usize, data: &[u8]) where for<'a> T: Deserialize<'a> {
        self.index = index;
        self.data = Some(decompress(data));
    }
}

pub struct RcCached<T, const CHUNK_ELEMS: usize>(RefCell<Cached<T, CHUNK_ELEMS>>);

impl<T, const CHUNK_ELEMS: usize> Default for RcCached<T, CHUNK_ELEMS> {
    fn default() -> Self {
        Self(RefCell::new(Default::default()))
    }
}

pub trait Cache: Default {
    fn is_cached(&self, _index: usize) -> bool { false }
    fn kill_all(&mut self) {}
}

impl Cache for Uncached {}

impl<'c, T, const CHUNK_ELEMS: usize> Cache for Cached<T, CHUNK_ELEMS> {
    fn is_cached(&self, index: usize) -> bool {
        self.data.is_some() && self.index == index
    }
    fn kill_all(&mut self) {
        *self = Default::default();
    }
}

impl<'c, T, const CHUNK_ELEMS: usize> Cache for RcCached<T, CHUNK_ELEMS> {
    fn is_cached(&self, index: usize) -> bool {
        self.0.borrow().is_cached(index)
    }
    fn kill_all(&mut self) {
        self.0.borrow_mut().kill_all();
    }
}

// pub trait CacheAccess<'c, T> {
//     type Item;
//     type IterItem;
//     type IterStorage;
//     fn get_uncompressed(x: &'c T) -> Self::Item;
//     fn get_compressed(&'c mut self, index: usize, offset: usize, data: &[u8]) -> Self::Item;
//     fn iter_storage(&'c mut self) -> Self::IterStorage;
// }

// impl<'c, T: 'c, const CHUNK_ELEMS: usize> CacheAccess<'c, T> for Cached<T, CHUNK_ELEMS>
// where for<'a> T: Deserialize<'a>
// {
//     type Item = &'c T;
//     type IterItem = T;
//     type IterStorage = RefCell<&'c mut Self>;
//     fn get_uncompressed(x: &'c T) -> Self::Item { x }
//     fn get_compressed(&'c mut self, index: usize, offset: usize, data: &[u8]) -> Self::Item {
//         if self.data.is_none() || self.index != index {
//             self.index = index;
//             self.data = Some(decompress(data));
//         }
//         &self.data.as_ref().unwrap().0[offset]
//     }
//     fn iter_storage(&'c mut self) -> Self::IterStorage {
//         RefCell::new(self)
//     }
// }

// impl<'c, T> CacheAccess<'c, T> for Uncached
// where for<'a> T: Deserialize<'a> + Clone {
//     type Item = T;
//     type IterItem = T;
//     type IterStorage = ();
//     fn get_uncompressed(x: &'c T) -> Self::Item { x.clone() }
//     fn get_compressed(&'c mut self, _index: usize, offset: usize, data: &[u8]) -> Self::Item {
//         let data: Vec<T> = decompress(&data);
//         data.into_iter().nth(offset).unwrap()
//     }
//     fn iter_storage(&'c mut self) -> Self::IterStorage {}
// }

impl<'de, T, const CHUNK_ELEMS: usize> Deserialize<'de> for CacheLine<T, CHUNK_ELEMS>
where T: for<'a> Deserialize<'a> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let data: Vec<T> = Deserialize::deserialize(deserializer)?;
        match data.into_boxed_slice().try_into() {
            Ok(data) => Ok(Self(data)),
            Err(err) => {
                let expected: &str = &format!("expected {} elements", CHUNK_ELEMS);
                Err(Error::invalid_length(err.len(), &expected))
            }
        }
    }
}

pub trait CacheAccess<T> {
    fn get_compressed(&mut self, index: usize, offset: usize, data: &[u8]) -> &T;
}

impl<T, const CHUNK_ELEMS: usize> CacheAccess<T> for Cached<T, CHUNK_ELEMS>
where for<'a> T: Deserialize<'a>
{
    fn get_compressed(&mut self, index: usize, offset: usize, data: &[u8]) -> &T {
        if !self.is_cached(index) {
            self.fill_cache(index, data);
        }
        &self.data.as_ref().unwrap().0[offset]
    }
}

pub trait RcCacheAccess<T, const CHUNK_ELEMS: usize> {
    fn get_compressed<'e>(&'e self, index: usize, offset: usize, data: &'e [u8]) -> Entry<'e, T, CHUNK_ELEMS>;
}

impl<T, const CHUNK_ELEMS: usize> RcCacheAccess<T, CHUNK_ELEMS> for RcCached<T, CHUNK_ELEMS>
where for<'a> T: Deserialize<'a>
{
    fn get_compressed<'e>(&'e self, index: usize, offset: usize, data: &'e [u8]) -> Entry<'e, T, CHUNK_ELEMS> {
        Entry::Compressed { cache: self, index, offset, data }
    }
}

pub enum Entry<'e, T, const CHUNK_ELEMS: usize> {
    Compressed {
        cache: &'e RcCached<T, CHUNK_ELEMS>,
        index: usize,
        offset: usize,
        data: &'e [u8],
    },
    Uncompressed(&'e T),
}

impl<'e, T, const CHUNK_ELEMS: usize> Entry<'e, T, CHUNK_ELEMS> {
    pub fn borrow(&self) -> EntryRef<'e, T, CHUNK_ELEMS> where for<'a> T: Deserialize<'a> {
        match *self {
            Entry::Compressed { cache, index, offset, data } => {
                let cache_ref = cache.0.borrow();
                let is_cached = cache_ref.is_cached(index);
                drop(cache_ref);
                if !is_cached {
                    let mut cache_ref = cache.0.borrow_mut();
                    cache_ref.fill_cache(index, data);
                    drop(cache_ref);
                }
                EntryRef::Compressed {
                    cache: cache.0.borrow(),
                    offset: offset,
                }
            }
            Entry::Uncompressed(data) => EntryRef::Uncompressed(data),
        }
    }
}

pub enum EntryRef<'e, T, const CHUNK_ELEMS: usize> {
    Compressed {
        cache: Ref<'e, Cached<T, CHUNK_ELEMS>>,
        offset: usize,
    },
    Uncompressed(&'e T),
}

impl<'e, T, const CHUNK_ELEMS: usize> std::ops::Deref for EntryRef<'e, T, CHUNK_ELEMS> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            EntryRef::Compressed { cache, offset } =>
                &cache.data.as_ref().unwrap().0[*offset],
            EntryRef::Uncompressed(data) => *data,
        }
    }
}
impl<'e, T, const CHUNK_ELEMS: usize> AsRef<T> for EntryRef<'e, T, CHUNK_ELEMS> {
    fn as_ref(&self) -> &T {
        use std::ops::Deref;
        Self::deref(self)
    }
}
