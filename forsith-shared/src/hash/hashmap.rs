use crate::bit::Bitmask;
use crate::hash::hashing::RandomState;
use core::hash::{BuildHasher, Hash};
use core::simd::Simd;
use forsith_base::buffer;
use forsith_base::buffer::Buffer;
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

    fn hash(&self, key: &K) -> u64 {
        self.hash_builder.hash_one(key)
    }

    fn probe(&self, h1: u64) -> Prober {
        Prober::new(h1, self)
    }

    unsafe fn store_group(&mut self, group_index: usize, group: Group) {
        let ptr = unsafe { self.content.as_mut_ptr().add(group_index).cast::<Tag>() };
        unsafe { group.store(ptr) };
    }

    unsafe fn get_key_value(&mut self, index: usize) -> &mut (K, V) {
        let base = unsafe { self.content.as_ptr().add(self.content.len()) };

        #[cfg(debug_assertions)]
        assert!(
            index * core::mem::size_of::<(K, V)>() < self.content.len(),
            "index {index} out of bounds for content length {}",
            self.content.len()
        );
        let kv_ptr = unsafe { base.sub(index * core::mem::size_of::<(K, V)>()) as *mut (K, V) };

        unsafe { &mut *kv_ptr }
    }

    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        let (h1, h2) = Self::split_hash(self.hash(&key));
        let mut deleted_index = None;

        let mut prober = self.probe(h1);
        while let Some(mut group) = prober.next(self) {
            for bit in group.tag_bitmask(Tag::entry(h2)) {
                println!("Found matching tag at bit: {bit}");
                let (k, v) = unsafe { self.get_key_value(prober.group_index() + bit as usize) };

                if *k == key {
                    return Some(core::mem::replace(v, val));
                }
            }

            if let Some(empty) = group.tag_bitmask(Tag::EMPTY).next_one() {
                println!("Found empty slot at bit: {empty}");
                let kv_index = deleted_index.unwrap_or_else(|| {
                    group.set_tag(empty as usize, Tag::entry(h2));
                    unsafe { self.store_group(prober.group_index(), group) };

                    prober.group_index() + empty as usize
                });

                let (k, v) = unsafe { self.get_key_value(kv_index) };
                *k = key;
                *v = val;
                return None;
            }

            if deleted_index.is_none()
                && let Some(deleted) = group.tag_bitmask(Tag::DELETED).next_one()
            {
                println!("Found deleted slot at bit: {deleted}");
                deleted_index = Some(deleted as usize + prober.group_index());
            }
        }

        unreachable!("Prober should never stop")
    }
}

pub struct Prober {
    accumulator: usize,
    group_index: usize,
}
impl Prober {
    #[allow(clippy::missing_panics_doc)]
    pub fn new<K: Hash + PartialEq, V, H: BuildHasher>(
        h1: u64,
        hashmap: &SwissTable<K, V, H>,
    ) -> Self {
        Self {
            accumulator: 1,
            group_index: usize::try_from(h1 & hashmap.bitmask as u64)
                .expect("not expecting a bitmask larger than usize"),
        }
    }

    #[must_use]
    pub const fn group_index(&self) -> usize {
        self.group_index * Group::SIZE
    }

    #[allow(clippy::missing_panics_doc)] // shouldn't panic if hashmap.bitmask is correct
    pub fn next<K: Hash + PartialEq, V, H: BuildHasher>(
        &mut self,
        hashmap: &SwissTable<K, V, H>,
    ) -> Option<Group> {
        self.group_index += self.accumulator;
        self.group_index &= hashmap.bitmask;
        self.accumulator += 1;

        #[cfg(debug_assertions)]
        assert!(
            self.group_index < hashmap.content.len(),
            "group_index {} out of bounds for content length {}",
            self.group_index,
            hashmap.content.len()
        );
        Some(unsafe { Group::load(hashmap.content.as_ptr().add(self.group_index()) as *mut Tag) })
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
pub struct Group(Simd<u8, 16>);
impl Group {
    const SIZE: usize = 16;

    /// # Safety
    /// ptr must be a valid pointer to a slice of length `Self::SIZE`
    #[must_use]
    pub const unsafe fn load(ptr: *mut Tag) -> Self {
        Self(Simd::from_slice(unsafe {
            &*ptr.cast::<[u8; Self::SIZE]>()
        }))
    }

    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn tag_bitmask(&self, tag: Tag) -> Bitmask {
        let mask = self.0.simd_eq(Simd::splat(tag.0));
        Bitmask::new(mask.to_bitmask().try_into().unwrap())
    }

    /// # Safety
    /// ptr must be a valid pointer to a slice of length `Self::SIZE`
    pub const unsafe fn store(self, ptr: *mut Tag) {
        unsafe {
            self.0.copy_to_slice(core::slice::from_raw_parts_mut(
                ptr.cast::<u8>(),
                Self::SIZE,
            ));
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tag_bitmask(Tag::EMPTY) != Bitmask::new(0)
    }

    pub fn set_tag(&mut self, index: usize, tag: Tag) {
        self.0[index] = tag.0;
    }

    #[must_use]
    pub fn is_full(&self) -> bool {
        self.0 & Simd::splat(0b100_0000) == Simd::splat(0)
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
    }
}
