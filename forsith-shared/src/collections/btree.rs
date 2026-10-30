use core::{fmt::Debug, mem::MaybeUninit, ops::Deref, ops::DerefMut, ptr::NonNull};

use forsith_proc::{Deref, DerefMut, IsVariant};

use crate::collections::inline::InlineArray;

const MAX_KEYS: u8 = 11;
const MIN_KEYS: u8 = MAX_KEYS / 2;
const MIDDLE: u8 = MIN_KEYS + 1;

pub struct Btree<K, V> {
    root: UnkownNode<K, V>,
    depth: u8,
}

impl<K: Debug, V: Debug> Debug for Btree<K, V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Btree")
            .field("depth", &self.depth)
            .field_with("root", |f| self.root.fmt(f, self.depth))
            .finish()
    }
}

impl<K, V> Btree<K, V> {
    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: Ord,
    {
        self.root.get(key, self.depth)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        if self.root.entries.len == MAX_KEYS {
            self.root.split_as_root(self.depth);
            self.depth += 1;
        }

        self.root.insert(key, value, self.depth)
    }

    pub fn remove(&mut self, key: &K) -> Option<V>
    where
        K: Ord,
    {
        self.root.remove(key, self.depth).map(|(_, v)| {
            if self.root.entries.len == 0 && self.depth > 0 {
                self.root.flatten_as_root(self.depth);
                self.depth -= 1;
            }

            v
        })
    }
}

impl<K, V> Drop for Btree<K, V> {
    fn drop(&mut self) {
        self.root.drop_inplace(self.depth);
        self.root.dealloc(1);
    }
}

impl<K, V> Default for Btree<K, V> {
    fn default() -> Self {
        Self {
            root: InternalNode::default().into(),
            depth: 0,
        }
    }
}

struct UnkownNode<K, V>(NonNull<LeafNode<K, V>>);

impl<K, V> Deref for UnkownNode<K, V> {
    type Target = LeafNode<K, V>;

    fn deref(&self) -> &LeafNode<K, V> {
        self.as_leaf()
    }
}

impl<K, V> DerefMut for UnkownNode<K, V> {
    fn deref_mut(&mut self) -> &mut LeafNode<K, V> {
        self.as_leaf_mut()
    }
}

impl<K, V> From<LeafNode<K, V>> for UnkownNode<K, V> {
    fn from(node: LeafNode<K, V>) -> Self {
        Self(Box::into_non_null(Box::new(node)))
    }
}

impl<K, V> From<InternalNode<K, V>> for UnkownNode<K, V> {
    fn from(node: InternalNode<K, V>) -> Self {
        unsafe {
            Self(NonNull::new_unchecked(
                Box::into_raw(Box::new(node)).cast::<LeafNode<K, V>>(),
            ))
        }
    }
}

impl<K, V> UnkownNode<K, V> {
    const fn as_leaf(&self) -> &LeafNode<K, V> {
        unsafe { self.0.as_ref() }
    }

    fn flatten_as_root(&mut self, depth: u8) {
        let internal = unsafe { self.as_internal_mut() };
        let mut new_root = internal.entries.take_child(ChildIdx::left(0));

        if depth == 1 {
            new_root.entries.drain_fully(&mut internal.entries, 0);
            new_root.dealloc(depth);
        } else {
            self.dealloc(depth);
            self.0 = new_root.0;
        }
    }

    fn remove_max(&mut self, depth: u8) -> (K, V) {
        let last = ChildIdx::right(self.entries.len - 1);

        if depth == 0 {
            return self.as_leaf_mut().entries.remove(last.idx()).unwrap();
        }

        let internal = unsafe { self.as_internal_mut() };
        let child = internal.entries.get_child_mut(last).unwrap();
        let max = child.remove_max(depth - 1);

        let last = ChildIdx::right(internal.entries.len - 1);
        internal.grow_if_needed(last, depth);

        max
    }

    const fn as_leaf_mut(&mut self) -> &mut LeafNode<K, V> {
        unsafe { self.0.as_mut() }
    }

    const unsafe fn as_internal_mut(&mut self) -> &mut InternalNode<K, V> {
        unsafe { self.0.cast::<InternalNode<K, V>>().as_mut() }
    }

    const unsafe fn as_internal(&self) -> &InternalNode<K, V> {
        unsafe { self.0.cast::<InternalNode<K, V>>().as_ref() }
    }

    fn split_right(&mut self, depth: u8) -> Self {
        match depth {
            0 => self.as_leaf_mut().split_right().into(),
            _ => unsafe { self.as_internal_mut().split_right().into() },
        }
    }

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>, depth: u8) -> core::fmt::Result
    where
        K: Debug,
        V: Debug,
    {
        match depth {
            0 => self.as_leaf().fmt(f),
            _ => unsafe { self.as_internal().fmt(depth, f) },
        }
    }

    fn get(&self, key: &K, depth: u8) -> Option<&V>
    where
        K: Ord,
    {
        match depth {
            0 => self.as_leaf().get(key),
            _ => unsafe { self.as_internal().get(key, depth) },
        }
    }

    fn insert(&mut self, key: K, value: V, depth: u8) -> Option<V>
    where
        K: Ord,
    {
        match depth {
            0 => self.as_leaf_mut().insert(key, value),
            _ => unsafe { self.as_internal_mut().insert(key, value, depth) },
        }
    }

    fn dealloc(&mut self, depth: u8) {
        unsafe {
            match depth {
                0 => drop(Box::from_non_null(self.0)),
                _ => drop(Box::from_raw(self.0.as_ptr().cast::<InternalNode<K, V>>())),
            }
        }
    }

    fn drop_inplace(&mut self, depth: u8) {
        match depth {
            0 => self.as_leaf_mut().entries.drop(),
            _ => unsafe { self.as_internal_mut().drop(depth) },
        }
    }

    fn drop(&mut self, depth: u8) {
        self.drop_inplace(depth);
        self.dealloc(depth);
    }

    fn split_as_root(&mut self, depth: u8)
    where
        K: Ord,
    {
        let (left, middle, right) = match depth {
            0 => self.as_leaf_mut().split_as_root(),
            _ => unsafe { self.as_internal_mut() }.split_as_root(),
        };

        let root = unsafe { self.as_internal_mut() };
        root.entries.len = 0;
        root.entries.set_child(ChildIdx::left(0), left);
        root.entries.insert(ChildIdx::right(0), middle, right);
    }

    fn remove(&mut self, key: &K, depth: u8) -> Option<(K, V)>
    where
        K: Ord,
    {
        match depth {
            0 => self.as_leaf_mut().remove(key),
            _ => unsafe { self.as_internal_mut().remove(key, depth) },
        }
    }
}

macro_rules! both {
    (($self:expr => $($tt:tt)*)) => {
        ($self.keys$($tt)* $self.values$($tt)*)
    };
    ($self:expr $(, $x:ident = $k:expr,$v:expr)* => $($tt:tt)*) => {{
        {
            $(let $x = $k;)*
            $self.keys$($tt)*
        }
        $(let $x = $v;)*
        $self.values$($tt)*
    }};
}

macro_rules! unsafe_both {
    ($($tt:tt)*) => {
        unsafe {
            both!($($tt)*)
        }
    };
}

struct Entries<K, V> {
    keys: InlineArray<K, { MAX_KEYS as usize }>,
    values: InlineArray<V, { MAX_KEYS as usize }>,
    len: u8,
}

impl<K, V> Default for Entries<K, V> {
    fn default() -> Self {
        Self {
            keys: unsafe { MaybeUninit::uninit().assume_init() },
            values: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }
}

impl<K, V> Entries<K, V> {
    fn drop(&mut self) {
        for i in 0..self.len {
            unsafe {
                both!(self => [i as usize].assume_init_drop(););
            }
        }
    }

    fn get(&self, idx: u8) -> Option<(&K, &V)> {
        (idx < self.len).then(|| unsafe_both!((self => .get(idx as usize),)))
    }

    fn get_mut(&mut self, idx: u8) -> Option<(&mut K, &mut V)> {
        (idx < self.len).then(|| unsafe_both!((self => .get_mut(idx as usize),)))
    }

    fn remove(&mut self, idx: u8) -> Option<(K, V)> {
        (idx < self.len && self.len > 0).then(|| {
            self.len -= 1;
            unsafe_both!((
                self => .shift_left(idx as usize..self.len as usize + 1, None),
            ))
        })
    }

    fn iter_keys(&self) -> impl Iterator<Item = &K> {
        self.keys
            .iter()
            .take(self.len as usize)
            .map(|k| unsafe { k.assume_init_ref() })
    }

    fn get_val(&self, idx: u8) -> Option<&V> {
        (idx < self.len).then(|| unsafe { self.values.get(idx as usize) })
    }

    fn drain_fully(&mut self, other: &mut Self, other_idx: u8) {
        self.drain(0, self.len, other, other_idx);
    }

    fn drain(&mut self, idx: u8, len: u8, other: &mut Self, other_idx: u8) {
        self.len -= len;
        other.allocate_slice(other_idx, len);

        both!(
            self,
            other=&mut other.keys,&mut other.values
            => .move_to_other(
                idx as usize..(idx + len) as usize,
                other,
                other_idx as usize
            );
        );
    }

    fn get_val_mut(&mut self, idx: u8) -> Option<&mut V> {
        (idx < self.len).then(|| unsafe { self.values.get_mut(idx as usize) })
    }

    fn replace(&mut self, idx: u8, (key, value): (K, V)) -> Option<(K, V)> {
        self.get_mut(idx)
            .map(|(k, v)| (core::mem::replace(k, key), core::mem::replace(v, value)))
    }

    fn insert(&mut self, idx: u8, (k, v): (K, V)) {
        self.len += 1;
        assert!(idx < self.len, "insert index out of bounds");
        both!(self, x = k,v => .shift_right_to_uninit(idx as usize..self.len as usize, x););
    }

    fn allocate_slice(&mut self, idx: u8, len: u8) {
        both!(self => .move_within(
            idx as usize..self.len as usize,
            (idx + len) as usize
        ));
        self.len += len;
    }
}

impl<K, V> Debug for Entries<K, V>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Entries")
            .field("len", &self.len)
            .field_with("keys", |f| unsafe { self.keys.fmt(f, self.len.into()) })
            .field_with("values", |f| unsafe { self.values.fmt(f, self.len.into()) })
            .finish()
    }
}

#[derive(Debug)]
#[repr(transparent)]
struct LeafNode<K, V> {
    entries: Entries<K, V>,
}

impl<K, V> Default for LeafNode<K, V> {
    fn default() -> Self {
        Self {
            entries: Entries::default(),
        }
    }
}

impl<K, V> LeafNode<K, V> {
    fn eq_key(&self, idx: u8, key: &K) -> bool
    where
        K: Ord,
    {
        self.entries.get(idx).is_some_and(|(k, _)| k == key)
    }

    fn remove(&mut self, key: &K) -> Option<(K, V)>
    where
        K: Ord,
    {
        let idx = self.partition_point(key);

        if !self.eq_key(idx, key) {
            return None;
        }

        Some(self.entries.remove(idx).unwrap())
    }

    fn get(&self, key: &K) -> Option<&V>
    where
        K: Ord,
    {
        self.entries
            .iter_keys()
            .position(|k| k == key)
            .map(|idx| self.entries.get_val(u8::try_from(idx).unwrap()).unwrap())
    }

    fn split_right(&mut self) -> Self {
        let mut right = Self::default();
        self.entries
            .drain(self.entries.len - MIN_KEYS, MIN_KEYS, &mut right.entries, 0);
        right
    }

    const fn is_full(&self) -> bool {
        self.entries.len == MAX_KEYS
    }

    fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        let idx = self.partition_point(&key);

        if self.eq_key(idx, &key) {
            return Some(core::mem::replace(
                self.entries.get_val_mut(idx).unwrap(),
                value,
            ));
        }

        self.entries.insert(idx, (key, value));

        None
    }

    fn partition_point(&self, key: &K) -> u8
    where
        K: Ord,
    {
        self.entries
            .iter_keys()
            .position(|k| k >= key)
            .unwrap_or(self.entries.len as usize)
            .try_into()
            .unwrap()
    }

    fn split_as_root(&mut self) -> (UnkownNode<K, V>, (K, V), UnkownNode<K, V>)
    where
        K: Ord,
    {
        let right = self.split_right();
        let middle = self.entries.remove(MIN_KEYS).unwrap();
        let left = core::mem::take(self);

        (left.into(), middle, right.into())
    }
}

#[repr(C)]
#[derive(Deref, DerefMut)]
struct ChildEntries<K, V> {
    #[deref]
    #[deref_mut]
    entries: Entries<K, V>,
    children: InlineArray<UnkownNode<K, V>, { MAX_KEYS as usize + 1 }>,
}

impl<K, V> Default for ChildEntries<K, V> {
    fn default() -> Self {
        Self {
            entries: Entries::default(),
            children: InlineArray::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, IsVariant, PartialEq, Eq)]
enum Side {
    Left,
    Right,
}

impl Side {
    const fn child_edge(self, len: u8) -> ChildIdx {
        match self {
            Self::Left => ChildIdx::left(0),
            Self::Right => ChildIdx::right(len - 1),
        }
    }

    const fn merged_child(self, idx: ChildIdx) -> ChildIdx {
        match self {
            Self::Left => idx.prev(),
            Self::Right => idx,
        }
    }

    const fn flipped(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChildIdx {
    idx: u8,
    side: Side,
}
impl core::ops::Add<u8> for ChildIdx {
    type Output = Self;

    fn add(self, rhs: u8) -> Self::Output {
        Self::new(self.idx + rhs, self.side)
    }
}

impl ChildIdx {
    const fn idx(self) -> u8 {
        self.idx
    }

    fn child_idx(self) -> u8 {
        self.idx + u8::from(self.side.is_right())
    }

    fn child_idx_usize(self) -> usize {
        self.idx as usize + usize::from(self.side.is_right())
    }

    const fn new(idx: u8, side: Side) -> Self {
        Self { idx, side }
    }

    const fn left(idx: u8) -> Self {
        Self::new(idx, Side::Left)
    }

    const fn right(idx: u8) -> Self {
        Self::new(idx, Side::Right)
    }

    const fn next_idx(self) -> Option<Self> {
        match self.side {
            Side::Left => None,
            Side::Right => Some(Self::new(self.idx + 1, Side::Left)),
        }
    }

    const fn next(self) -> Self {
        match self.side {
            Side::Left => Self::new(self.idx, Side::Right),
            Side::Right => Self::new(self.idx + 1, Side::Right),
        }
    }

    const fn prev(self) -> Self {
        match self.side {
            Side::Left => Self::new(self.idx - 1, Side::Left),
            Side::Right => Self::new(self.idx, Side::Left),
        }
    }

    const fn with(self, side: Side) -> Self {
        Self::new(self.idx, side)
    }

    const fn sibling(self, side: Side) -> Self {
        match side {
            Side::Left => self.prev(),
            Side::Right => self.next(),
        }
    }
}

impl<K, V> ChildEntries<K, V> {
    const fn len(&self) -> u8 {
        self.entries.len
    }

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>, depth: u8) -> core::fmt::Result
    where
        K: Debug,
        V: Debug,
    {
        f.debug_struct("ChildEntries")
            .field_with("entries", |f| self.entries.fmt(f))
            .field_with("children", |f| {
                f.debug_list()
                    .entry_with(|f| {
                        for i in 0..=self.entries.len {
                            self.get_child(ChildIdx::left(i))
                                .unwrap()
                                .fmt(f, depth - 1)?;
                        }
                        Ok(())
                    })
                    .finish()
            })
            .finish()
    }

    fn get_children_disjoint_mut<const N: usize>(
        &mut self,
        idxs: [ChildIdx; N],
    ) -> Option<[&mut UnkownNode<K, V>; N]> {
        self.children
            .get_disjoint_mut(idxs.map(ChildIdx::child_idx_usize))
            .ok()
            .map(|arr| unsafe { arr.map(|c| c.assume_init_mut()) })
    }

    fn get_child(&self, idx: ChildIdx) -> Option<&UnkownNode<K, V>> {
        (idx.child_idx() <= self.len())
            .then(|| unsafe { self.children.get(idx.child_idx() as usize) })
    }

    fn get_child_mut(&mut self, idx: ChildIdx) -> Option<&mut UnkownNode<K, V>> {
        (idx.child_idx() <= self.len())
            .then(|| unsafe { self.children.get_mut(idx.child_idx() as usize) })
    }

    fn set_child(&mut self, idx: ChildIdx, child: UnkownNode<K, V>) {
        assert!(idx.child_idx() <= self.len(), "child index out of bounds");

        self.children.set(idx.child_idx_usize(), child);
    }

    fn remove(&mut self, idx: ChildIdx) -> (UnkownNode<K, V>, (K, V)) {
        (
            unsafe {
                self.children
                    .shift_left(idx.child_idx() as usize..self.len() as usize + 1, None)
            },
            self.entries.remove(idx.idx()).unwrap(),
        )
    }

    fn take_child(&mut self, idx: ChildIdx) -> UnkownNode<K, V> {
        unsafe { self.children.remove(idx.child_idx_usize()) }
    }

    fn insert(&mut self, idx: ChildIdx, kv: (K, V), child: UnkownNode<K, V>) {
        self.entries.insert(idx.idx(), kv);

        self.children
            .shift_right_to_uninit(idx.child_idx_usize()..self.entries.len as usize + 1, child);
    }

    fn drain(&mut self, idx: ChildIdx, len: u8, other: &mut Self, other_idx: ChildIdx) {
        self.entries
            .drain(idx.idx(), len, &mut other.entries, other_idx.idx());

        self.drain_children(idx, len, other, other_idx);
    }

    fn drain_fully(&mut self, other: &mut Self, other_idx: ChildIdx) {
        self.drain(ChildIdx::left(0), self.len(), other, other_idx);
    }

    fn drain_children(&mut self, idx: ChildIdx, len: u8, other: &mut Self, other_idx: ChildIdx) {
        let old_child_len = other.entries.len - len + 1;
        let len = len + u8::from(other_idx.side.is_left());

        other.children.move_within(
            other_idx.child_idx() as usize..old_child_len as usize,
            (other_idx.child_idx() + len) as usize,
        );

        self.children.move_to_other(
            idx.child_idx() as usize..(idx.child_idx() + len) as usize,
            &mut other.children,
            other_idx.child_idx() as usize,
        );
    }

    fn children_mut(&mut self) -> &mut [UnkownNode<K, V>] {
        unsafe { self.children.get_mut(0..=self.len() as usize) }
    }
}

#[repr(C)]
struct InternalNode<K, V> {
    entries: ChildEntries<K, V>,
}

impl<K, V> Default for InternalNode<K, V> {
    fn default() -> Self {
        Self {
            entries: ChildEntries::default(),
        }
    }
}

impl<K, V> Deref for InternalNode<K, V> {
    type Target = LeafNode<K, V>;

    fn deref(&self) -> &Self::Target {
        self.as_node()
    }
}

impl<K, V> InternalNode<K, V> {
    const fn as_node(&self) -> &LeafNode<K, V> {
        unsafe { core::mem::transmute(self) }
    }

    fn fmt(&self, depth: u8, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    where
        K: Debug,
        V: Debug,
    {
        f.debug_struct("InternalNode")
            .field_with("entries", |f| self.entries.fmt(f, depth))
            .finish()
    }

    fn get(&self, key: &K, depth: u8) -> Option<&V>
    where
        K: Ord,
    {
        let idx = self.partition_point(key);

        if self.eq_key(idx.idx(), key) {
            return Some(self.entries.get_val(idx.idx()).unwrap());
        }

        let child = self.entries.get_child(idx).unwrap();
        child.get(key, depth - 1)
    }

    fn partition_point(&self, key: &K) -> ChildIdx
    where
        K: Ord,
    {
        ChildIdx::left(self.as_node().partition_point(key))
    }

    fn remove_predecessor(&mut self, idx: ChildIdx, depth: u8) -> (K, V) {
        let left_child = self.entries.get_child_mut(idx).unwrap();
        left_child.remove_max(depth - 1)
    }

    fn swap_remove_kv(&mut self, idx: ChildIdx, depth: u8) -> (K, V) {
        let pred = self.remove_predecessor(idx, depth);
        let kv = self.entries.replace(idx.idx(), pred).unwrap();

        self.grow_if_needed(idx, depth);

        kv
    }

    fn grow_if_needed(&mut self, idx: ChildIdx, depth: u8) -> ChildIdx {
        let child_len = self.entries.get_child(idx).unwrap().entries.len;

        if child_len < MIN_KEYS {
            self.grow_child(idx, depth)
        } else {
            idx
        }
    }

    fn remove(&mut self, key: &K, depth: u8) -> Option<(K, V)>
    where
        K: Ord,
    {
        let idx = self.partition_point(key);

        if self.eq_key(idx.idx(), key) {
            return Some(self.swap_remove_kv(idx, depth));
        }
        let kv = self
            .entries
            .get_child_mut(idx)
            .unwrap()
            .remove(key, depth - 1);

        let child_idx = self.grow_if_needed(idx, depth);

        let child = self.entries.get_child(child_idx).unwrap();
        assert!(
            child.entries.len >= MIN_KEYS,
            "child must have at least MIN_KEYS after growing"
        );

        kv
    }

    fn biggest_sibling(&mut self, idx: ChildIdx) -> (Side, &mut UnkownNode<K, V>) {
        self.siblings(idx)
            .into_iter()
            .enumerate()
            .filter_map(|(i, s)| s.map(|s| (i, s)))
            .max_by_key(|(_, s)| s.entries.len)
            .map(|(i, s)| {
                let side = match i {
                    0 => Side::Left,
                    1 => Side::Right,
                    _ => unreachable!(),
                };
                (side, s)
            })
            .expect("at least one sibling must exist")
    }

    fn grow_child(&mut self, idx: ChildIdx, depth: u8) -> ChildIdx {
        let (side, sibling) = self.biggest_sibling(idx);

        if sibling.entries.len > MIN_KEYS {
            self.grow_from_sibling(idx, side, depth);
            idx
        } else {
            self.merge_child(idx, side, depth)
        }
    }

    fn siblings(&mut self, idx: ChildIdx) -> [Option<&mut UnkownNode<K, V>>; 2] {
        if idx.child_idx_usize() == 0 {
            return [None, Some(self.entries.get_child_mut(idx.next()).unwrap())];
        } else if idx.child_idx_usize() == self.entries.len() as usize {
            return [Some(self.entries.get_child_mut(idx.prev()).unwrap()), None];
        }

        self.entries
            .get_children_disjoint_mut([idx.prev(), idx.next()])
            .map(|[left, right]| [Some(left), Some(right)])
            .unwrap()
    }

    fn split_right(&mut self) -> Self {
        let mut right = Self {
            entries: ChildEntries {
                entries: self.as_node_mut().split_right().entries,
                children: InlineArray::default(),
            },
        };

        self.entries.drain_children(
            ChildIdx::left(MIDDLE),
            MIN_KEYS,
            &mut right.entries,
            ChildIdx::left(0),
        );

        right
    }

    fn split_child(&mut self, idx: ChildIdx, depth: u8) {
        let child = self.entries.get_child_mut(idx).unwrap();
        let right = child.split_right(depth - 1);
        let middle = child.entries.remove(MIN_KEYS).unwrap();

        self.entries.insert(idx.next(), middle, right);
    }

    fn replace_val(&mut self, idx: u8, value: V) -> Option<V> {
        self.entries
            .get_val_mut(idx)
            .map(|v| core::mem::replace(v, value))
    }

    fn insert(&mut self, key: K, value: V, depth: u8) -> Option<V>
    where
        K: Ord,
    {
        let mut idx = self.partition_point(&key);

        if self.eq_key(idx.idx(), &key) {
            return Some(self.replace_val(idx.idx(), value).unwrap());
        }

        if self.is_child_full(idx) {
            self.split_child(idx, depth);

            idx = self.partition_point(&key);
            if self.eq_key(idx.idx(), &key) {
                return Some(self.replace_val(idx.idx(), value).unwrap());
            }
        }

        self.entries
            .get_child_mut(idx)
            .unwrap()
            .insert(key, value, depth - 1)
    }

    fn is_child_full(&self, idx: ChildIdx) -> bool {
        self.entries.get_child(idx).unwrap().is_full()
    }

    fn drop(&mut self, depth: u8) {
        self.entries.drop();

        self.entries
            .children_mut()
            .iter_mut()
            .for_each(|c| c.drop(depth - 1));
    }

    fn split_as_root(&mut self) -> (UnkownNode<K, V>, (K, V), UnkownNode<K, V>)
    where
        K: Ord,
    {
        let right = self.split_right();
        let middle = self.as_node_mut().entries.remove(MIN_KEYS).unwrap();
        let left = core::mem::take(&mut *self);

        (left.into(), middle, right.into())
    }

    fn as_node_mut(&mut self) -> &mut LeafNode<K, V> {
        unsafe { &mut *(&raw mut *self).cast::<LeafNode<K, V>>() }
    }

    fn merge_child(&mut self, idx: ChildIdx, side: Side, depth: u8) -> ChildIdx {
        let idx = side.merged_child(idx);
        let (mut sibling, kv) = self.entries.remove(idx.with(side));
        let child = self.entries.get_child_mut(idx).unwrap();

        let mut dest = side.child_edge(child.entries.len + 1);

        child.entries.insert(dest.idx(), kv);
        if side.is_right() {
            dest = dest.next_idx().unwrap();
        }

        if depth == 1 {
            sibling.entries.drain_fully(&mut child.entries, dest.idx());
        } else {
            let child = unsafe { child.as_internal_mut() };
            let sibling = unsafe { sibling.as_internal_mut() };

            sibling.entries.drain_fully(&mut child.entries, dest);
        }

        sibling.dealloc(depth - 1);
        idx
    }

    fn grow_from_sibling(&mut self, idx: ChildIdx, side: Side, depth: u8) {
        let child_len = self.entries.get_child(idx).unwrap().entries.len;
        let child_add = side.child_edge(child_len + 1);
        let splitting_kv = side.merged_child(idx).idx();

        let sibling = self.entries.get_child_mut(idx.sibling(side)).unwrap();
        let sibling_remove = side.flipped().child_edge(sibling.entries.len);

        if depth == 1 {
            let kv = sibling.entries.remove(sibling_remove.idx()).unwrap();
            let kv = self.entries.replace(splitting_kv, kv).unwrap();

            let child = self.entries.get_child_mut(idx).unwrap();

            child.entries.insert(child_add.idx(), kv);
        } else {
            let sibling = unsafe { sibling.as_internal_mut() };

            let (c, kv) = sibling.entries.remove(sibling_remove);
            let kv = self.entries.replace(splitting_kv, kv).unwrap();

            let child = unsafe { self.entries.get_child_mut(idx).unwrap().as_internal_mut() };

            child.entries.insert(child_add, kv, c);
        }
    }
}

#[cfg(test)]
mod btree_tests {
    use super::*;
    use crate::rng::Rng;
    use crate::rng::SimpleRng;

    #[test]
    fn test_btree_insert_and_get() {
        let mut btree = Btree::default();

        for i in 0..100 {
            btree.insert(i, i * 10);
        }

        for i in 0..100 {
            assert_eq!(btree.get(&i), Some(&(i * 10)));
        }

        assert_eq!(btree.get(&100), None);
    }

    #[test]
    fn test_btree_insert_duplicate() {
        let mut btree = Btree::default();

        assert_eq!(btree.insert(1, 10), None);
        assert_eq!(btree.get(&1), Some(&10));

        assert_eq!(btree.insert(1, 20), Some(10));
        assert_eq!(btree.get(&1), Some(&20));
    }

    #[test]
    fn test_empty_btree_get() {
        let btree: Btree<u64, u64> = Btree::default();
        assert_eq!(btree.get(&1), None);
    }

    #[test]
    fn test_btree_drop_safety() {
        let mut btree = Btree::default();

        for i in 0..100 {
            btree.insert(i.to_string(), (i * 10).to_string());
        }

        for i in 0..99 {
            assert_eq!(btree.get(&i.to_string()), Some(&(i * 10).to_string()));
        }

        drop(btree);
    }

    #[test]
    fn test_btree_insert_and_remove() {
        let mut btree = Btree::default();

        for i in 0..100 {
            btree.insert(i, i * 10);
        }

        for i in 0..100 {
            assert_eq!(btree.remove(&i), Some(i * 10));
            assert_eq!(btree.get(&i), None);
        }
    }

    #[test]
    fn test_btree_insert_and_remove_random() {
        let mut btree = Btree::default();

        let keys: Vec<u64> = (0..1000).map(|_| SimpleRng::random()).collect();

        for &key in &keys {
            btree.insert(key, key);
        }

        for &key in &keys {
            assert_eq!(btree.remove(&key), Some(key));
            assert_eq!(btree.get(&key), None);
        }
    }

    #[test]
    fn test_btree_insert_and_get_random() {
        let mut btree = Btree::default();

        let keys: Vec<u64> = (0..1000).map(|_| SimpleRng::random()).collect();

        for &key in &keys {
            btree.insert(key, key);
        }

        for &key in &keys {
            assert_eq!(btree.get(&key), Some(&(key)));
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn random_operation_btree_compared_to_std() {
        let mut btree = Btree::default();
        let mut std = std::collections::BTreeMap::new();

        for _ in 0..10000 {
            let key = SimpleRng::random() % 255;
            let value = SimpleRng::random();

            match SimpleRng::random() % 3 {
                0 => assert_eq!(btree.insert(key, value), std.insert(key, value)),
                1 => assert_eq!(btree.remove(&key), std.remove(&key)),
                2 => assert_eq!(btree.get(&key), std.get(&key)),
                _ => unreachable!(),
            }
        }
    }
}
