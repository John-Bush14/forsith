use core::{
    fmt::Debug,
    mem::MaybeUninit,
    ops::Deref,
    ptr::NonNull,
};

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
        let v = self.root.remove(key, self.depth);

        if self.root.entries.len == 0 && self.depth > 0 {
            self.flatten();
        }

        v.map(|(_, v)| v)
    }

    fn flatten(&mut self) {
        let mut new_root = unsafe {
            self.root
                .assume_internal_mut()
                .entries
                .take_child(ChildIdx::left(0))
        };

        if self.depth == 1 {
            let len = new_root.entries.len;
            new_root
                .entries
                .drain_into(0, len, &mut self.root.entries, 0);
            new_root.dealloc(0);
        } else {
            self.root.dealloc(self.depth);
            self.root = new_root;
        }

        self.depth -= 1;
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
            root: UnkownNode::from_internal(InternalNode::default()),
            depth: 0,
        }
    }
}

struct UnkownNode<K, V>(NonNull<LeafNode<K, V>>);

impl<K, V> core::ops::Deref for UnkownNode<K, V> {
    type Target = LeafNode<K, V>;

    fn deref(&self) -> &LeafNode<K, V> {
        self.as_leaf()
    }
}

impl<K, V> core::ops::DerefMut for UnkownNode<K, V> {
    fn deref_mut(&mut self) -> &mut LeafNode<K, V> {
        self.as_leaf_mut()
    }
}

impl<K, V> core::convert::From<LeafNode<K, V>> for UnkownNode<K, V> {
    fn from(node: LeafNode<K, V>) -> Self {
        Self::from_leaf(node)
    }
}

impl<K, V> core::convert::From<InternalNode<K, V>> for UnkownNode<K, V> {
    fn from(node: InternalNode<K, V>) -> Self {
        Self::from_internal(node)
    }
}

impl<K, V> UnkownNode<K, V> {
    const fn as_leaf(&self) -> &LeafNode<K, V> {
        unsafe { self.0.as_ref() }
    }

    fn remove_rightmost_keyval(&mut self, depth: u8) -> (K, V) {
        let len = self.entries.len;

        if depth == 0 {
            self.as_leaf_mut().entries.remove(len - 1).unwrap()
        } else {
            let child = unsafe {
                self.assume_internal_mut()
                    .entries
                    .get_child_mut(ChildIdx::left(len))
                    .unwrap()
            };

            let kv = child.remove_rightmost_keyval(depth - 1);

            if child.entries.len < MIN_KEYS {
                let len = self.entries.len;
                unsafe { self.assume_internal_mut() }.grow_child(ChildIdx::left(len), depth);
            }

            kv
        }
    }

    const fn as_leaf_mut(&mut self) -> &mut LeafNode<K, V> {
        unsafe { self.0.as_mut() }
    }

    const unsafe fn assume_internal_mut(&mut self) -> &mut InternalNode<K, V> {
        unsafe { self.0.cast::<InternalNode<K, V>>().as_mut() }
    }

    const unsafe fn assume_internal(&self) -> &InternalNode<K, V> {
        unsafe { self.0.cast::<InternalNode<K, V>>().as_ref() }
    }

    fn split_right(&mut self, depth: u8) -> Self {
        match depth {
            0 => self.as_leaf_mut().split_right().into(),
            _ => unsafe { self.assume_internal_mut().split_right().into() },
        }
    }

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>, depth: u8) -> core::fmt::Result
    where
        K: Debug,
        V: Debug,
    {
        match depth {
            0 => self.as_leaf().fmt(f),
            _ => unsafe { self.assume_internal().fmt(depth, f) },
        }
    }

    fn get(&self, key: &K, depth: u8) -> Option<&V>
    where
        K: Ord,
    {
        match depth {
            0 => self.as_leaf().get(key),
            _ => unsafe { self.assume_internal().get(key, depth) },
        }
    }

    fn insert(&mut self, key: K, value: V, depth: u8) -> Option<V>
    where
        K: Ord,
    {
        match depth {
            0 => self.as_leaf_mut().insert(key, value),
            _ => unsafe { self.assume_internal_mut().insert(key, value, depth) },
        }
    }

    fn from_leaf(node: LeafNode<K, V>) -> Self {
        Self(Box::into_non_null(Box::new(node)))
    }

    fn from_internal(node: InternalNode<K, V>) -> Self {
        unsafe {
            Self(NonNull::new_unchecked(
                Box::into_raw(Box::new(node)).cast::<LeafNode<K, V>>(),
            ))
        }
    }

    fn dealloc(&mut self, depth: u8) {
        if depth == 0 {
            unsafe {
                let _ = Box::from_raw(self.0.as_ptr());
            }
        } else {
            unsafe {
                let _ = Box::from_raw(self.0.as_ptr().cast::<InternalNode<K, V>>());
            }
        }
    }

    fn drop_inplace(&mut self, depth: u8) {
        if depth == 0 {
            self.as_leaf_mut().entries.drop();
        } else {
            unsafe {
                self.assume_internal_mut().drop_children(depth);
            }
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
            _ => unsafe { self.assume_internal_mut() }.split_as_root(),
        };

        let root = unsafe { self.assume_internal_mut() };
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
            _ => unsafe { self.assume_internal_mut().remove(key, depth) },
        }
    }
}

macro_rules! both {
    (($self:expr => $($tt:tt)*)) => {
        ($self.keys$($tt)* $self.values$($tt)*)
    };
    ($self:expr $(, $other:ident)? $(; $x:ident = $k:ident,$v:ident)? => $($tt:tt)*) => {{
        {
            $(let $other = &mut $other.keys;)?
            $(let $x = $k;)?
            $self.keys$($tt)*
        }
        $(let $other = &mut $other.values;)?
        $(let $x = $v;)?
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

    fn drain_into(&mut self, idx: u8, len: u8, other: &mut Self, other_idx: u8) {
        self.len -= len;
        other.allocate_slice(other_idx, len);

        both!(self, other => .move_to_other(
            idx as usize..(idx + len) as usize,
            other,
            other_idx as usize
        ););
    }

    fn get_val_mut(&mut self, idx: u8) -> Option<&mut V> {
        (idx < self.len).then(|| unsafe { self.values.get_mut(idx as usize) })
    }

    fn replace(&mut self, idx: u8, key: K, value: V) -> Option<(K, V)> {
        self.get_mut(idx)
            .map(|(k, v)| (core::mem::replace(k, key), core::mem::replace(v, value)))
    }

    fn insert(&mut self, idx: u8, key: K, value: V) {
        assert!(idx <= self.len, "insert index out of bounds");
        self.len += 1;
        both!(self; x = key,value => .shift_right_to_uninit(idx as usize..self.len as usize, x););
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
    fn idx_eq_key(&self, idx: u8, key: &K) -> bool
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

        if !self.idx_eq_key(idx, key) {
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
            .drain_into(self.entries.len - MIN_KEYS, MIN_KEYS, &mut right.entries, 0);
        right
    }

    fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        let idx = self.partition_point(&key);

        if self.idx_eq_key(idx, &key) {
            return Some(core::mem::replace(
                self.entries.get_val_mut(idx).unwrap(),
                value,
            ));
        }

        self.entries.insert(idx, key, value);

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
        let left = std::mem::take(self);

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

#[derive(Debug, Clone, Copy, IsVariant)]
enum Side {
    Left,
    Right,
}
#[derive(Debug, Clone, Copy)]
struct ChildIdx {
    idx: u8,
    side: Side,
}
impl ChildIdx {
    fn idx(&self) -> u8 {
        self.idx
    }

    fn side(&self) -> Side {
        self.side
    }

    fn children_idx_u8(&self) -> u8 {
        self.idx + u8::from(self.side.is_right())
    }

    fn children_idx(&self) -> usize {
        self.idx as usize + usize::from(self.side.is_right())
    }

    fn new(idx: u8, side: Side) -> Self {
        Self { idx, side }
    }

    fn left(idx: u8) -> Self {
        Self::new(idx, Side::Left)
    }

    fn right(idx: u8) -> Self {
        Self::new(idx, Side::Right)
    }

    fn next(&self) -> Self {
        match self.side {
            Side::Left => Self::new(self.idx, Side::Right),
            Side::Right => Self::new(self.idx + 1, Side::Left),
        }
    }

    fn add(&self, i: u8) -> ChildIdx {
        Self::new(self.idx + i, self.side)
    }
}

impl<K, V> ChildEntries<K, V> {
    fn len(&self) -> u8 {
        self.entries.len
    }

    fn get_child(&self, idx: ChildIdx) -> Option<&UnkownNode<K, V>> {
        (idx.children_idx_u8() <= self.entries.len)
            .then(|| unsafe { self.children.get(idx.children_idx()) })
    }

    fn get_child_mut(&mut self, idx: ChildIdx) -> Option<&mut UnkownNode<K, V>> {
        (idx.children_idx_u8() <= self.entries.len)
            .then(|| unsafe { self.children.get_mut(idx.children_idx()) })
    }

    fn set_child(&mut self, idx: ChildIdx, child: UnkownNode<K, V>) {
        self.children.set(idx.children_idx(), child)
    }

    fn remove(&mut self, idx: ChildIdx) -> (UnkownNode<K, V>, (K, V)) {
        (
            unsafe {
                self.children
                    .shift_left(idx.children_idx()..self.len() as usize + 1, None)
            },
            self.entries.remove(idx.idx()).unwrap(),
        )
    }

    fn take_child(&mut self, idx: ChildIdx) -> UnkownNode<K, V> {
        unsafe { self.children.remove(idx.children_idx()) }
    }

    fn insert(&mut self, idx: ChildIdx, (k, v): (K, V), child: UnkownNode<K, V>) {
        self.entries.insert(idx.idx(), k, v);

        self.children
            .shift_right_to_uninit(idx.children_idx()..self.entries.len as usize + 1, child);
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
    fn as_node(&self) -> &LeafNode<K, V> {
        unsafe { &*(self as *const Self as *const LeafNode<K, V>) }
    }

    fn fmt(&self, _depth: u8, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    where
        K: Debug,
        V: Debug,
    {
        f.debug_struct("InternalNode").finish()
    }

    fn get(&self, key: &K, depth: u8) -> Option<&V>
    where
        K: Ord,
    {
        let idx = self.partition_point(key);

        if self.idx_eq_key(idx.idx(), key) {
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

    fn remove(&mut self, key: &K, depth: u8) -> Option<(K, V)>
    where
        K: Ord,
    {
        let idx = self.partition_point(key);

        if self.idx_eq_key(idx.idx(), key) {
            let left_child = self.entries.get_child_mut(idx).unwrap();
            let (pred_key, pred_value) = left_child.remove_rightmost_keyval(depth - 1);
            let kv = self
                .entries
                .replace(idx.idx(), pred_key, pred_value)
                .unwrap();

            let left_child = self.entries.get_child_mut(idx).unwrap();
            if left_child.entries.len < MIN_KEYS {
                self.grow_child(idx, depth);
            }

            return Some(kv);
        }
        let kv = self
            .entries
            .get_child_mut(idx)
            .unwrap()
            .remove(key, depth - 1);
        let child_len = self.entries.get_child(idx).unwrap().entries.len;

        let child_idx = if child_len < MIN_KEYS {
            self.grow_child(idx, depth)
        } else {
            idx
        };

        let child = self.entries.get_child(child_idx).unwrap();
        assert!(
            child.entries.len >= MIN_KEYS,
            "child must have at least MIN_KEYS after growing"
        );

        kv
    }

    fn grow_child(&mut self, idx: ChildIdx, depth: u8) -> ChildIdx {
        let idx = ChildIdx::left(
            (idx.children_idx() as u8).min(self.entries.len),
        );
        let child_len = self.entries.get_child(idx).unwrap().entries.len;

        let Some((side, Some(sibling))) = self
            .child_siblings(idx)
            .into_iter()
            .enumerate()
            .max_by_key(|(_, s)| s.as_ref().map_or(0, |s| s.entries.len))
        else {
            unreachable!("child should have a sibling");
        };

        let side = match side {
            0 => Side::Left,
            1 => Side::Right,
            _ => unreachable!(),
        };

        let child_add = match side {
            Side::Left => ChildIdx::left(0),
            Side::Right => ChildIdx::right(child_len),
        };

        let splitting_key = idx.children_idx_u8() - u8::from(side.is_left());

        if sibling.entries.len > MIN_KEYS {
            let sibling_remove = match side {
                Side::Left => ChildIdx::right(sibling.entries.len - 1),
                Side::Right => ChildIdx::left(0),
            };

            if depth - 1 == 0 {
                let kv = sibling.entries.remove(sibling_remove.idx()).unwrap();

                let kv = self
                    .entries
                    .replace(splitting_key, kv.0, kv.1)
                    .unwrap();

                let child = self.entries.get_child_mut(idx).unwrap();

                child.entries.insert(child_add.idx(), kv.0, kv.1);
            } else {
                let sibling = unsafe { sibling.assume_internal_mut() };
                let (c, (k, v)) = sibling.entries.remove(sibling_remove);

                let kv = self.entries.replace(splitting_key, k, v).unwrap();

                let child = unsafe {
                    self.entries
                        .get_child_mut(idx)
                        .unwrap()
                        .assume_internal_mut()
                };
                child.entries.insert(child_add, kv, c);
            }
            idx
        } else {
            let (mut sibling, (k, v)) = self.entries.remove(ChildIdx::new(splitting_key, side));
            let child_idx = match side {
                Side::Left => ChildIdx::left(idx.idx() - 1),
                Side::Right => idx,
            };
            let child = self.entries.get_child_mut(child_idx).unwrap();

            if depth - 1 == 0 {
                child.entries.insert(child_add.idx(), k, v);

                let len = sibling.entries.len;
                sibling.entries.drain_into(
                    0,
                    len,
                    &mut child.entries,
                    child_add.idx() + u8::from(side.is_right()),
                );
            } else {
                let child = unsafe { child.assume_internal_mut() };
                let sibling = unsafe { sibling.assume_internal_mut() };
                let sibling_len = sibling.entries.len;

                let c = sibling
                    .entries
                    .take_child(ChildIdx::left(sibling_len));
                child.entries.insert(child_add, (k, v), c);

                let child_add = match child_add.side() {
                    Side::Left => ChildIdx::left(child_add.idx()),
                    Side::Right => ChildIdx::left(child_add.idx() + 1),
                };

                for i in 0..sibling_len {
                    let (c, kv) = sibling.entries.remove(ChildIdx::left(0));
                    child.entries.insert(child_add.add(i), kv, c);
                }
            }

            sibling.dealloc(depth - 1);
            match side {
                Side::Left => ChildIdx::left(idx.idx() - 1),
                Side::Right => idx,
            }
        }
    }

    fn child_siblings(&mut self, idx: ChildIdx) -> [Option<&mut UnkownNode<K, V>>; 2] {
        if idx.children_idx() == 0 {
            [None, Some(self.entries.get_child_mut(idx.next()).unwrap())]
        } else if idx.children_idx() == self.entries.len() as usize {
            [
                (idx.idx() > 0)
                    .then(|| self.entries.get_child_mut(ChildIdx::left(idx.idx() - 1)).unwrap()),
                None,
            ]
        } else {
            self.entries
                .children
                .get_disjoint_mut([idx.children_idx() - 1, idx.children_idx() + 1])
                .map(|[left, right]| {
                    [
                        Some(unsafe { left.assume_init_mut() }),
                        Some(unsafe { right.assume_init_mut() }),
                    ]
                })
                .unwrap()
        }
    }

    fn split_right(&mut self) -> Self {
        let mut right = Self {
            entries: ChildEntries {
                entries: self.as_node_mut().split_right().entries,
                children: Default::default(),
            },
        };

        for i in 0..=MIN_KEYS as u8 {
            right.entries.set_child(
                ChildIdx::left(i),
                self.entries.take_child(ChildIdx::left(MIDDLE + i)),
            );
        }

        right
    }

    fn insert(&mut self, key: K, value: V, depth: u8) -> Option<V>
    where
        K: Ord,
    {
        let mut idx = self.partition_point(&key);

        if self.idx_eq_key(idx.idx(), &key) {
            return Some(core::mem::replace(
                self.entries.get_val_mut(idx.idx()).unwrap(),
                value,
            ));
        }

        let child = self.entries.get_child_mut(idx).unwrap();

        if child.entries.len == MAX_KEYS {
            let right = child.split_right(depth - 1);
            let middle = child.entries.remove(MIN_KEYS).unwrap();

            self.entries.insert(idx.next(), middle, right);
            idx = self.partition_point(&key);

            if self.idx_eq_key(idx.idx(), &key) {
                return Some(core::mem::replace(
                    self.entries.get_val_mut(idx.idx()).unwrap(),
                    value,
                ));
            }
        }

        self.entries
            .get_child_mut(idx)
            .unwrap()
            .insert(key, value, depth - 1)
    }

    fn drop_children(&mut self, depth: u8) {
        self.entries.drop();

        for i in 0..=self.entries.len {
            self.entries
                .get_child_mut(ChildIdx::left(i))
                .unwrap()
                .drop(depth - 1);
        }
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
