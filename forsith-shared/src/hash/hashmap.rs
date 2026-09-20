use crate::bit::Bitmask;
use crate::hash::hashing::RandomState;
use core::hash::{BuildHasher, Hash};
use core::simd::Simd;
use core::simd::cmp::SimdPartialEq;
use forsith_base::buffer;
use forsith_base::buffer::Buffer;

#[allow(type_alias_bounds)]
pub type HashMap<K: Hash, V, H = RandomState> = SwissTable<K, V, H>;

pub struct SwissTable<K: Hash + PartialEq, V, H: BuildHasher = RandomState> {
    hash_builder: H,
    content: Buffer<u8>,
    bitmask: usize,
    growth_left: usize,
    _key: core::marker::PhantomData<K>,
    _value: core::marker::PhantomData<V>,
}

impl<K: Hash + PartialEq, V, H: Default + BuildHasher> Default for SwissTable<K, V, H> {
    fn default() -> Self {
        Self::new(H::default())
    }
}

// public interface
impl<K: Hash + PartialEq, V, H: BuildHasher> SwissTable<K, V, H> {
    pub fn new(hash_builder: H) -> Self {
        Self {
            hash_builder,
            content: buffer![Tag::EMPTY.byte(); 0],
            bitmask: 0,
            growth_left: 0,
            _key: core::marker::PhantomData,
            _value: core::marker::PhantomData,
        }
    }

    #[must_use]
    pub const fn growth_left(&self) -> usize {
        self.growth_left
    }

    pub fn with_capacity(hash_builder: H, capacity: usize) -> Self {
        let mut table = Self::new(hash_builder);
        table.realloc_buffer(capacity);
        table
    }

    /// Resize the hash table to the nearest multiple of 16 and power of 2 greater than or equal to
    /// `new_capacity`. Rehashes all existing entries into the new table.
    pub fn resize(&mut self, new_capacity: usize) {
        let old_capacity = self.capacity();
        let old_content = self.realloc_buffer(new_capacity);

        self.rehash_from(&old_content, old_capacity);
    }

    pub fn capacity(&self) -> usize {
        self.content.len() / (1 + core::mem::size_of::<(K, V)>())
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let (h1, h2) = Self::split_hash(self.hash(key));

        let mut prober = self.probe(h1)?;
        loop {
            let group = prober.next(self);

            for bit in group.bitmask(Tag::entry(h2)) {
                let (k, v) = self.get_key_value(prober.kv_index(bit));

                if *k == *key {
                    return Some(v);
                }
            }

            if group.bitmask(Tag::EMPTY) != Bitmask::EMPTY {
                return None;
            }
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        self.grow_if_needed();

        let (h1, h2) = Self::split_hash(self.hash(&key));
        let mut deleted_index = None;

        let mut prober = self
            .probe(h1)
            .expect("Should never be empty as just grew if needed");

        loop {
            let group = prober.next(self);

            for bit in group.bitmask(Tag::entry(h2)) {
                let (k, v) = self.get_key_value_mut(prober.kv_index(bit));

                if *k == key {
                    return Some(core::mem::replace(v, val));
                }
            }

            if let Some(empty) = group.bitmask(Tag::EMPTY).next_one() {
                let index = deleted_index.unwrap_or_else(|| {
                    self.growth_left -= 1;

                    prober.group_index(empty)
                });

                self.set_tag(index, Tag::entry(h2));
                *self.get_key_value_mut(index.into()) = (key, val);

                return None;
            }

            if deleted_index.is_none()
                && let Some(deleted) = group.bitmask(Tag::DELETED).next_one()
            {
                deleted_index = Some(prober.group_index(deleted));
            }
        }
    }
}

// private interface
impl<K: Hash + PartialEq, V, H: BuildHasher> SwissTable<K, V, H> {
    const MAX_LOAD_FACTOR: u8 = 85;

    fn realloc_buffer(&mut self, new_capacity: usize) -> Buffer<u8> {
        let capacity = Self::choose_capacity(new_capacity);
        self.set_bitmask(capacity);
        self.growth_left = capacity * Self::MAX_LOAD_FACTOR as usize / 100;

        core::mem::replace(
            &mut self.content,
            buffer![Tag::EMPTY.byte(); capacity * (1 + core::mem::size_of::<(K, V)>())],
        )
    }

    fn rehash_from(&mut self, old_content: &Buffer<u8>, old_capacity: usize) {
        for (i, group) in Group::iter_all(old_capacity, old_content).enumerate() {
            for bit in !(group.bitmask(Tag::EMPTY) | group.bitmask(Tag::DELETED)) {
                self.growth_left -= 1;

                let index = GroupIndex::new(i, bit);
                let h2 = index.get_tag(old_content).h2();

                let kv = KvIndex::<K, V>::from_group_index(index).get_kv(old_content);
                self.insert_rehash(kv, h2);
            }
        }
    }

    fn choose_capacity(capacity: usize) -> usize {
        capacity.next_power_of_two().max(Group::SIZE)
    }

    const fn set_bitmask(&mut self, capacity: usize) {
        let groups = capacity / Group::SIZE;
        let needed_bits = groups.bit_width() - 1;
        self.bitmask = (1 << needed_bits) - 1;
    }

    fn insert_rehash(&mut self, kv: &(K, V), h2: u8) {
        let (h1, _) = Self::split_hash(self.hash(&kv.0));

        let mut prober = self
            .probe(h1)
            .expect("Should always have a prober when rehashing into a larger table");
        loop {
            let group = prober.next(self);

            if let Some(empty) = group.bitmask(Tag::EMPTY).next_one() {
                let group_index = prober.group_index(empty);
                self.set_tag(group_index, Tag::entry(h2));

                let kvnew = self.get_key_value_mut(group_index.into());
                unsafe {
                    core::ptr::copy_nonoverlapping(kv, kvnew, 1);
                }

                return;
            }
        }
    }

    fn split_hash(hash: u64) -> (u64, u8) {
        (
            hash.wrapping_shr(7),
            u8::try_from(hash & ((1 << 7) - 1)).unwrap(),
        )
    }

    fn hash(&self, key: &K) -> u64 {
        self.hash_builder.hash_one(key)
    }

    fn probe(&self, h1: u64) -> Option<Prober> {
        if self.capacity() == 0 {
            return None;
        }

        Some(Prober::new(h1, self))
    }

    fn set_tag(&mut self, index: GroupIndex, tag: Tag) {
        index.set_tag(&mut self.content, tag);
    }

    fn get_key_value(&self, index: KvIndex<K, V>) -> &(K, V) {
        index.get_kv(&self.content)
    }

    fn get_key_value_mut(&mut self, index: KvIndex<K, V>) -> &mut (K, V) {
        index.get_kv_mut(&mut self.content)
    }

    fn load_group(&self, group_index: usize) -> Group {
        Group::load_from(&self.content, group_index)
    }

    fn grow_if_needed(&mut self) {
        if self.growth_left == 0 {
            self.resize(self.capacity() * 2);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct GroupIndex(usize);
impl GroupIndex {
    #[must_use]
    pub const fn new(group_index: usize, bit: u8) -> Self {
        Self(group_index * Group::SIZE + bit as usize)
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }

    pub fn get_tag(self, content: &[u8]) -> Tag {
        Tag::from(content[self.index()])
    }

    pub const fn set_tag(self, content: &mut [u8], tag: Tag) {
        content[self.index()] = tag.byte();
    }
}

impl<K, V> From<GroupIndex> for KvIndex<K, V> {
    fn from(index: GroupIndex) -> Self {
        Self::from_group_index(index)
    }
}

impl<K, V> Clone for KvIndex<K, V> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K, V> Copy for KvIndex<K, V> {}

#[derive(PartialEq, Eq)]
struct KvIndex<K, V>(usize, core::marker::PhantomData<(K, V)>);
impl<K, V> KvIndex<K, V> {
    const KV_SIZE: usize = core::mem::size_of::<(K, V)>();

    #[must_use]
    pub const fn new(group_index: usize, bit: u8) -> Self {
        Self(
            (group_index * Group::SIZE + bit as usize + 1) * Self::KV_SIZE,
            core::marker::PhantomData,
        )
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }

    pub const fn from_group_index(index: GroupIndex) -> Self {
        Self(
            (index.index() + 1) * Self::KV_SIZE,
            core::marker::PhantomData,
        )
    }

    pub fn get_kv(self, content: &[u8]) -> &(K, V) {
        let start = content.len() - self.index();
        let slice = &content[start..start + Self::KV_SIZE];
        unsafe { &*(slice.as_ptr().cast::<(K, V)>()) }
    }

    pub fn get_kv_mut(self, content: &mut [u8]) -> &mut (K, V) {
        let start = content.len() - self.index();
        let slice = &mut content[start..start + Self::KV_SIZE];
        unsafe { &mut *(slice.as_mut_ptr().cast::<(K, V)>()) }
    }
}

struct Prober {
    accumulator: usize,
    group_start: usize,
}
impl Prober {
    #[allow(clippy::missing_panics_doc)]
    pub fn new<K: Hash + PartialEq, V, H: BuildHasher>(
        h1: u64,
        hashmap: &SwissTable<K, V, H>,
    ) -> Self {
        Self {
            accumulator: 1,
            group_start: usize::try_from(h1 & hashmap.bitmask as u64)
                .expect("not expecting a bitmask larger than usize"),
        }
    }

    #[must_use]
    pub const fn group_index(&self, bit: u8) -> GroupIndex {
        GroupIndex::new(self.group_start, bit)
    }

    pub const fn kv_index<K, V>(&self, bit: u8) -> KvIndex<K, V> {
        KvIndex::new(self.group_start, bit)
    }

    #[allow(clippy::missing_panics_doc)] // shouldn't panic if hashmap.bitmask is correct
    pub fn next<K: Hash + PartialEq, V, H: BuildHasher>(
        &mut self,
        hashmap: &SwissTable<K, V, H>,
    ) -> Group {
        self.group_start += self.accumulator;
        self.group_start &= hashmap.bitmask;
        self.accumulator += 1;

        hashmap.load_group(self.group_start)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Tag(u8);

impl From<u8> for Tag {
    fn from(byte: u8) -> Self {
        Self::from_byte(byte)
    }
}

impl From<Tag> for u8 {
    fn from(tag: Tag) -> Self {
        tag.byte()
    }
}

impl Tag {
    pub const EMPTY: Self = Self(0b1111_1111);
    pub const DELETED: Self = Self(0b1111_1110);

    /// # Panics
    /// Panics if the byte is not a valid tag (i.e. if the highest bit is set and the byte is not
    /// EMPTY or DELETED)
    #[must_use]
    pub fn from_byte(byte: u8) -> Self {
        #[cfg(debug_assertions)]
        assert!(
            !(byte >= 0b1000_0000 && !(byte == Self::EMPTY.byte() || byte == Self::DELETED.byte())),
            "Tried to create a tag from a byte that is not a valid tag: {byte:#010b}"
        );

        Self(byte)
    }

    #[must_use]
    pub const fn byte(&self) -> u8 {
        self.0
    }

    /// # Panics
    /// Panics if 'h2' takes more than 7 bits
    #[must_use]
    pub const fn entry(h2: u8) -> Self {
        #[cfg(debug_assertions)]
        assert!(h2 < 0b1000_0000, "h2 must be less than 0b1000_0000");

        Self(h2)
    }

    /// # Panics
    /// Panics if tag is empty or deleted (i.e. if the highest bit is set)
    #[must_use]
    pub const fn h2(&self) -> u8 {
        #[cfg(debug_assertions)]
        assert!(
            self.0 < 0b1000_0000,
            "Tried to get h2 from a tag that is not a valid entry"
        );

        self.0
    }
}

// Represents a group of 16 tags in the hash map
struct Group(Simd<u8, 16>);
impl Group {
    const SIZE: usize = 16;

    fn iter_all(capacity: usize, content: &[u8]) -> impl Iterator<Item = Group> + '_ {
        (0..capacity / Self::SIZE).map(move |i| Self::load_from(content, i))
    }

    #[must_use]
    const fn load(slice: &[u8]) -> Self {
        Self(Simd::from_slice(slice))
    }

    #[must_use]
    pub fn load_from(slice: &[u8], index: usize) -> Self {
        let start = index * Self::SIZE;
        let end = start + Self::SIZE;
        Self::load(&slice[start..end])
    }

    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn bitmask(&self, tag: Tag) -> Bitmask {
        let mask = self.0.simd_eq(Simd::splat(tag.0));
        Bitmask::new(mask.to_bitmask().try_into().unwrap())
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

        for i in 0..1 << 24 {
            table.insert(i, i * 4);
        }

        let duration = std::time::Instant::now();
        for i in 0..1 << 24 {
            table.get(&i);
        }

        panic!("Time taken to get 1 million entries: {:?}", duration.elapsed());
    }
}
