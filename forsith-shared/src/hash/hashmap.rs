use crate::bit::Bitmask;
use crate::hash::hashing::RandomState;
use core::hash::{BuildHasher, Hash};
use core::simd::Simd;
use forsith_base::buffer;
use forsith_base::buffer::Buffer;
use std::simd::cmp::SimdPartialEq;

pub type HashMap<K: Hash, V, H = RandomState> = SwissTable<K, V, H>;

pub struct SwissTable<K: Hash + PartialEq, V, H: BuildHasher = RandomState> {
    hash_builder: H,
    content: Buffer<u8>,
    bitmask: usize,
    capacity: usize,
    empty: usize,
    _key: core::marker::PhantomData<K>,
    _value: core::marker::PhantomData<V>,
}

impl<K: Hash + PartialEq, V, H: BuildHasher> SwissTable<K, V, H> {
    const MAX_LOAD_FACTOR: u8 = 85;

    pub fn new(hash_builder: H) -> Self {
        Self {
            hash_builder,
            content: buffer![Tag::EMPTY.0; 0],
            bitmask: 0,
            capacity: 0,
            empty: 0,
            _key: core::marker::PhantomData,
            _value: core::marker::PhantomData,
        }
    }

    unsafe fn load_group(&self, group_index: usize) -> Group {
        unsafe {Group::load(self.content.as_ptr().add(group_index * Group::SIZE).cast::<Tag>())}
    }

    /// in percent, 0-100
    ///
    /// # Panics
    /// Panics if empty slots are greater than capacity
    pub fn load_factor(&self) -> u8 {
        if self.capacity() == 0 {return 100;}

        100u8 - u8::try_from(self.empty * 100 / self.capacity()).expect("Empty slots should be less than capacity")
    }

    fn grow_if_needed(&mut self) {
        if self.load_factor() > Self::MAX_LOAD_FACTOR {
            let old_capacity = self.capacity();
            self.set_capacity(self.capacity() * 2);
            println!("Growing table from {} to {}", old_capacity, self.capacity());
            let capacity = self.capacity();

            let content = core::mem::replace(&mut self.content, buffer![Tag::EMPTY.0; capacity + capacity * core::mem::size_of::<(K, V)>()]);
            let groups = (0..old_capacity / Group::SIZE).map(|i| unsafe {
                #[cfg(debug_assertions)]
                assert!(i * Group::SIZE < content.len(), "group index {} out of bounds for content length {}", i * Group::SIZE, content.len());
                Group::load(content.as_ptr().add(i * Group::SIZE) as *mut Tag)
            });

            self.empty = capacity;
            for (i, group) in groups.enumerate() {
                for bit in !(group.bitmask(Tag::EMPTY) | group.bitmask(Tag::DELETED)) {
                    self.empty -= 1;

                    let index = i * Group::SIZE + bit as usize;

                    let h2 = content[index];

                    let kv = unsafe {content.as_ptr().add(content.len() - (index+1) * core::mem::size_of::<(K, V)>()).cast::<(K, V)>()};

                    self.insert_rehash(kv, h2);
                }
            }
        }
    }

    fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity.next_power_of_two().max(Group::SIZE);

        let groups = self.capacity() / Group::SIZE;
        let needed_bits = groups.bit_width() - 1;
        self.bitmask = (1 << needed_bits) - 1;
    }

    pub const fn capacity(&self) -> usize {self.capacity}

    fn split_hash(hash: u64) -> (u64, u8) {
        (hash, u8::try_from(hash & ((1 << 7) - 1)).unwrap())
    }

    fn hash(&self, key: &K) -> u64 {
        self.hash_builder.hash_one(key)
    }

    fn probe(&self, h1: u64) -> Option<Prober> {
        if self.capacity() < 16 {
            return None;
        }

        Some(Prober::new(h1, self))
    }

    unsafe fn set_tag(&mut self, group_index: usize, bit: usize, tag: Tag) {
        unsafe { self.content.as_mut_ptr().add(group_index * Group::SIZE + bit).cast::<Tag>().write(tag) };
    }

    unsafe fn get_key_value(&self, group_index: usize, bit: usize) -> &(K, V) {
        let index = group_index * Group::SIZE + bit + 1;

        unsafe { & *self.content.as_ptr().add(self.content.len() - index * core::mem::size_of::<(K, V)>()).cast::<(K, V)>() }
    }

    unsafe fn get_key_value_mut(&mut self, group_index: usize, bit: usize) -> &mut (K, V) {
        let index = group_index * Group::SIZE + bit + 1;

        unsafe { &mut *self.content.as_mut_ptr().add(self.content.len() - index * core::mem::size_of::<(K, V)>()).cast::<(K, V)>() }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let (h1, h2) = Self::split_hash(self.hash(key));

        let mut prober = self.probe(h1)?;
        while let Some(group) = prober.next(self) {
            for bit in group.bitmask(Tag::entry(h2)) {
                let (k, v) = unsafe { self.get_key_value(prober.group_index(), bit as usize) };

                if *k == *key {
                    return Some(v);
                }
            }

            if group.bitmask(Tag::EMPTY) != Bitmask::EMPTY {
                return None;
            }
        }

        None
    }

    fn insert_rehash(&mut self, kv: *const (K, V), h2: u8) {
        let (h1, _) = Self::split_hash(self.hash(unsafe { &(*kv).0 }));

        let mut prober = self.probe(h1).expect("Should always have a prober when rehashing into a larger table");
        while let Some(group) = prober.next(self) {
            if let Some(empty) = group.bitmask(Tag::EMPTY).next_one() {
                unsafe {
                    self.set_tag(prober.group_index(), empty as usize, Tag::entry(h2));
                    let kvnew = core::ptr::from_mut::<(K, V)>(self.get_key_value_mut(prober.group_index(), empty as usize));
                    core::ptr::copy_nonoverlapping(kv, kvnew, 1);
                }
                return;
            }
        }

        unreachable!("prober should never stop")
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        self.grow_if_needed();

        let (h1, h2) = Self::split_hash(self.hash(&key));
        let mut deleted_index = None;

        let mut prober = self.probe(h1).expect("Should never be empty as just grew if needed");
        while let Some(group) = prober.next(self) {
            for bit in group.bitmask(Tag::entry(h2)) {
                let (k, v) = unsafe { self.get_key_value_mut(prober.group_index(), bit as usize) };

                if *k == key {
                    return Some(core::mem::replace(v, val));
                }
            }

            if let Some(empty) = group.bitmask(Tag::EMPTY).next_one() {
                let kv_index = deleted_index.unwrap_or_else(|| {
                    self.empty -= 1;

                    (prober.group_index(), empty as usize)
                });

                unsafe {
                    self.set_tag(kv_index.0, kv_index.1, Tag::entry(h2));
                }

                let (k, v) = unsafe { self.get_key_value_mut(kv_index.0, kv_index.1) };
                *k = key;
                *v = val;
                return None;
            }

            if deleted_index.is_none()
                && let Some(deleted) = group.bitmask(Tag::DELETED).next_one()
            {
                deleted_index = Some((deleted as usize, prober.group_index()));
            }
        }

        unreachable!("prober should never stop")
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
        self.group_index
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
        Some(unsafe { hashmap.load_group(self.group_index) })
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
    pub const unsafe fn load(ptr: *const Tag) -> Self {
        Self(Simd::from_slice(unsafe {
            &*ptr.cast::<[u8; Self::SIZE]>()
        }))
    }

    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn bitmask(&self, tag: Tag) -> Bitmask {
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
        self.bitmask(Tag::EMPTY) != Bitmask::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_test() {
        let mut table = SwissTable::<u32, u32>::new(RandomState::default());
        table.insert(1u32, 10);
        table.insert(2u32, 20);
        table.insert(3u32, 30);

        for i in 4..32u32 {
            assert_eq!(table.get(&i), None);
        }

        for i in 0..32u32 {
            table.insert(i, i * 2);
            assert_eq!(table.get(&i), Some(&(i * 2)));
        }

        for i in 0..32u32 {
            assert_eq!(table.get(&i), Some(&(i * 2)));
        }

        for i in 0..32u32 {
            assert_eq!(table.insert(i, i * 3), Some(i * 2));
        }

        for i in 0..1 << 10 {
            table.insert(i, i * 4);
            assert_eq!(table.get(&i), Some(&(i * 4)));
        }
    }
}
