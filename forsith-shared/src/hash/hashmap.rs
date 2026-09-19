use forsith_base::buffer::{Buffer};
use forsith_base::buffer;

use crate::bit::Bitmask;
use crate::hash::hashing::{DefaultHasher, RandomState};
use core::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher};
use core::simd::Simd;
use std::simd::cmp::SimdPartialEq;

pub type HashMap<K: Hash, V, H = RandomState> = SwissTable<K, V, H>;

pub struct SwissTable<K: Hash + PartialEq, V, H: BuildHasher = RandomState> {
    // Internal representation of the hash map
    // This is a placeholder; actual implementation would depend on the chosen data structure
    // For example, it could be a vector of buckets, each containing a linked list of key-value pairs
    hash_builder: H,
    content: Buffer<u8>,
    bitmask: usize,
    _key: core::marker::PhantomData<K>,
    _value: core::marker::PhantomData<V>,
}

impl<K: Hash + PartialEq, V, H: BuildHasher> SwissTable<K, V, H> {
    pub fn new(hash_builder: H) -> Self {
        Self {
            hash_builder,
            content: buffer![Tag::EMPTY.0; (16 + 16 * 2) * 2],
            bitmask: 1,
            _key: core::marker::PhantomData,
            _value: core::marker::PhantomData,
        }
    }

    fn split_hash(hash: u64) -> (u64, u8) {
        (hash, u8::try_from(hash & ((1 << 7) - 1)).unwrap())
    }

    fn hash(&self, key: &K) -> u64 {self.hash_builder.hash_one(key)}

    fn probe(&self, h1: u64) -> Prober<'_, K, V, H> {
        Prober::new(self, h1)
    }

    #[allow(clippy::mut_from_ref)]
    unsafe fn get_key_value(&self, index: usize) -> &mut (K, V) {
        let base = unsafe {self.content.as_ptr().add(self.content.len())};

        #[cfg(debug_assertions)]
        assert!(index * core::mem::size_of::<(K, V)>() < self.content.len(), "index {index} out of bounds for content length {}", self.content.len());
        let kv_ptr = unsafe {base.sub(index * core::mem::size_of::<(K, V)>()) as *mut (K, V)};

        unsafe {&mut *kv_ptr}
    }

    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        let (h1, h2) = Self::split_hash(self.hash(&key));
        let mut deleted_index = None;

        let mut prober = self.probe(h1);
        while let Some(mut group) = prober.next() {
            for bit in group.tag_bitmask(Tag::entry(h2)) {
                println!("Found matching tag at bit: {}", bit);
                let (k, v) = unsafe {self.get_key_value(prober.group_index() + bit as usize)};

                if *k == key {
                    return Some(core::mem::replace(v, val));
                }
            }

            if let Some(empty) = group.tag_bitmask(Tag::EMPTY).next_one() {
                println!("Found empty slot at bit: {}", empty);
                let kv_index = deleted_index.unwrap_or_else(|| {
                    group.set_tag(empty as usize, Tag::entry(h2));
                    prober.group_index() + empty as usize
                });

                let (k, v) = unsafe {self.get_key_value(kv_index)};
                *k = key;
                *v = val;
                return None;
            }

            if deleted_index.is_none() && let Some(deleted) = group.tag_bitmask(Tag::DELETED).next_one() {
                println!("Found deleted slot at bit: {}", deleted);
                deleted_index = Some(deleted as usize + prober.group_index());
            }
        }

        todo!()
    }
}

pub struct Prober<'a, K: Hash + PartialEq, V, H: BuildHasher> {
    accumulator: usize,
    hashmap: &'a SwissTable<K, V, H>,
    group_index: usize,
}
impl<'a, K: Hash + PartialEq, V, H: BuildHasher> Prober<'a, K, V, H> {
    #[allow(clippy::missing_panics_doc)]
    pub fn new(hashmap: &'a SwissTable<K, V, H>, h1: u64) -> Self {
        Self {
            accumulator: 1,
            group_index: usize::try_from(h1 & hashmap.bitmask as u64).expect("not expecting a bitmask larger than usize"),
            hashmap,
        }
    }

    #[must_use]
    pub const fn group_index(&self) -> usize {
        self.group_index * Group::SIZE
    }
}

impl<K: Hash + PartialEq, V, H: BuildHasher> Iterator for Prober<'_, K, V, H> {
    type Item = Group;

    fn next(&mut self) -> Option<Self::Item> {
        self.group_index += self.accumulator;
        self.group_index &= self.hashmap.bitmask;
        self.accumulator += 1;

        #[cfg(debug_assertions)]
        assert!(self.group_index < self.hashmap.content.len(), "group_index {} out of bounds for content length {}", self.group_index, self.hashmap.content.len());
        Some(unsafe { Group::load(self.hashmap.content.as_ptr().add(self.group_index()) as *mut Tag) })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Tag(u8);

impl Tag {
    pub const EMPTY: Self = Self(0b1111_1111);
    pub const DELETED: Self = Self(0b1111_1110);

    /// # Panics
    /// Panics if 'h2' takes more than 7 bits
    #[must_use]
    pub const fn entry(h2: u8) -> Self {
        #[cfg(debug_assertions)]
        assert!(h2 < 0b1000_0000, "h2 must be less than 0b1000_0000");
        Self(h2)
    }
}

// Represents a group of 16 tags in the hash map
pub struct Group(*mut Tag);
impl Group {
    const SIZE: usize = 16;

    /// # Safety
    /// ptr must be a valid pointer to a slice of length `Self::SIZE`
    #[must_use]
    pub const unsafe fn load(ptr: *mut Tag) -> Self {
        Self(ptr)
    }

    const fn as_simd(&self) -> Simd<u8, 16> {
        Simd::from_slice(unsafe { &*self.0.cast::<[u8; Self::SIZE]>() })
    }

    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn tag_bitmask(&self, tag: Tag) -> Bitmask {
        let mask = self.as_simd().simd_eq(Simd::splat(tag.0));
        Bitmask::new(mask.to_bitmask().try_into().unwrap())
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tag_bitmask(Tag::EMPTY) != Bitmask::new(0)
    }

    pub fn set_tag(&mut self, index: usize, tag: Tag) {
        self.tag_mut(index).0 = tag.0;
    }

    pub fn tag_mut(&mut self, index: usize) -> &mut Tag {
        let ptr = unsafe { self.0.add(index) };
        unsafe { &mut *ptr }
    }

    #[must_use]
    pub fn is_full(&self) -> bool {
        self.as_simd() & Simd::splat(0b100_0000) == Simd::splat(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_test() {
        let mut table = SwissTable::<u8, u8>::new(RandomState::default());
        table.insert(17, 10);
        table.insert(2, 20);
        table.insert(3, 30);

        for i in 0..32 {
            table.insert(i, i * 2);
        }

        for i in 0..32 {
            assert_eq!(table.insert(i, i * 3), Some(i * 2));
        }

        panic!()
    }
}
