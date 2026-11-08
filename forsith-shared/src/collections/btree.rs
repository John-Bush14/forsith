use crate::collections::inline::InlineArray;
#[allow(unused_imports)]
use core::cmp::Ordering;
use core::{
    fmt::{Debug, Display, Write},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};
use forsith_base::string::StringExt;
use forsith_proc::{Deref, DerefMut, IsVariant};

const MAX_KEYS: u8 = 11;
const MIN_KEYS: u8 = MAX_KEYS / 2;
const MIDDLE: u8 = MIN_KEYS + 1;

pub struct Btree<K, V> {
    root: RootNode<K, V>,
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

impl<K: Display, V: Display> Display for Btree<K, V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Btree \n")?;
        f.write_str(&self.root.display(self.depth))
    }
}

impl<K, V> Btree<K, V> {
    #[allow(clippy::missing_panics_doc)]
    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: Ord,
    {
        let mut cursor = self.open_cursor();
        cursor.find(key)?;
        Some(cursor.keyval_longlived().unwrap().1)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        self.split_if_needed();
        self.open_cursor_mut().insert(key, value)
    }

    #[cfg(test)]
    fn assert_invariants(&self)
    where
        K: Ord,
    {
        self.root.assert_invariants(self.depth);
    }

    pub fn remove(&mut self, key: &K) -> Option<V>
    where
        K: Ord,
    {
        let mut cursor = self.open_cursor_mut();
        cursor.find(key)?;
        let v = Some(cursor.remove().1);
        self.flatten_if_needed();
        v
    }

    fn open_cursor(&self) -> Cursor<'_, Immut, K, V> {
        unsafe { Cursor::open(self.root.clone(), ChildIdx::START, self.depth) }
    }

    fn open_cursor_mut(&mut self) -> Cursor<'_, Mut, K, V> {
        unsafe { Cursor::open(self.root.clone(), ChildIdx::START, self.depth) }
    }

    #[allow(dead_code)]
    fn min(&self) -> Cursor<'_, Immut, K, V>
    where
        K: Ord,
    {
        let mut cursor = self.open_cursor();
        cursor.descend_to_leaf_with(IdxChooser::Left);
        cursor
    }

    #[allow(dead_code)]
    fn max(&self) -> Cursor<'_, Immut, K, V>
    where
        K: Ord,
    {
        let mut cursor = self.open_cursor();
        cursor.descend_to_leaf_with(IdxChooser::Right);
        cursor
    }

    fn flatten_if_needed(&mut self) {
        if self.root.entries.len == 0 && self.depth > 0 {
            self.root.flatten(self.depth);
            self.depth -= 1;
        }
    }

    fn split_if_needed(&mut self)
    where
        K: Ord,
    {
        if self.root.entries.len == MAX_KEYS {
            self.root.split(self.depth);
            self.depth += 1;
        }
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
            root: RootNode::default(),
            depth: 0,
        }
    }
}

pub struct NodeCursor<'btree, K, V> {
    node: UnkownNode<K, V>,
    depth: u8,
    _phantom: core::marker::PhantomData<&'btree ()>,
}

impl<K, V> NodeCursor<'_, K, V> {
    const fn open(node: UnkownNode<K, V>, depth: u8) -> Self {
        Self {
            node,
            depth,
            _phantom: core::marker::PhantomData,
        }
    }

    fn ascend(&mut self) -> Option<()>
    where
        K: Ord,
    {
        let parent = unsafe { self.node.parent?.as_mut() };
        self.node = parent.into();
        self.depth += 1;

        Some(())
    }

    fn descend(&mut self, idx: ChildIdx) {
        assert!(self.depth > 0, "cannot descend from a leaf node");

        self.node = unsafe { self.node.as_internal_mut() }
            .entries
            .get_child_mut(idx)
            .unwrap()
            .clone();

        self.depth -= 1;
    }
}

struct Mut;
struct Immut;

struct Cursor<'btree, M, K, V> {
    node: NodeCursor<'btree, K, V>,
    idx: ChildIdx,
    _phantom: core::marker::PhantomData<&'btree M>,
}

impl<'btree, K, V> Cursor<'btree, Immut, K, V> {
    const fn node_longlived(&self) -> &'btree UnkownNode<K, V> {
        unsafe { self.node.node.reborrow() }
    }

    fn keyval_longlived(&self) -> Option<(&'btree K, &'btree V)> {
        self.node_longlived().entries.get(self.idx.idx())
    }
}

enum IdxChooser<'a, K: Ord> {
    Left,
    Right,
    Partition(&'a K),
}

impl<K: Ord> Clone for IdxChooser<'_, K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: Ord> Copy for IdxChooser<'_, K> {}

impl<K: Ord> IdxChooser<'_, K> {
    fn choose<V>(&self, node: &UnkownNode<K, V>) -> ChildIdx {
        match self {
            Self::Left => ChildIdx::START,
            Self::Right => ChildIdx::right(node.entries.len - 1),
            Self::Partition(key) => ChildIdx::left(node.partition_point(key)),
        }
    }
}

impl<M, K, V> Cursor<'_, M, K, V> {
    const unsafe fn open(node: UnkownNode<K, V>, idx: ChildIdx, depth: u8) -> Self {
        Self {
            node: NodeCursor::open(node, depth),
            idx,
            _phantom: core::marker::PhantomData,
        }
    }

    const fn node(&self) -> &UnkownNode<K, V> {
        &self.node.node
    }

    fn keyval(&self) -> Option<(&K, &V)> {
        self.node().entries.get(self.idx.idx())
    }

    fn ascend(&mut self) -> Option<()>
    where
        K: Ord,
    {
        // ascend shouldn't mutate anynodeg
        let (key, _) = unsafe { self.node().reborrow() }.entries.get(0)?;

        self.node.ascend()?;
        self.idx = unsafe { self.node().as_internal() }.partition_point(key);

        Some(())
    }

    fn descend_if_possible(&mut self, chooser: IdxChooser<'_, K>) -> bool
    where
        K: Ord,
    {
        if self.node.depth > 0 {
            self.descend(chooser);
            return true;
        }

        false
    }

    fn choose_idx(&mut self, chooser: IdxChooser<'_, K>)
    where
        K: Ord,
    {
        self.idx = chooser.choose(self.node());
    }

    fn descend_with(&mut self, chooser: IdxChooser<'_, K>)
    where
        K: Ord,
    {
        self.choose_idx(chooser);
        self.descend(chooser);
    }

    fn descend(&mut self, chooser: IdxChooser<'_, K>)
    where
        K: Ord,
    {
        self.node.descend(self.idx);
        self.choose_idx(chooser);
    }

    fn descend_to_leaf_with(&mut self, chooser: IdxChooser<'_, K>)
    where
        K: Ord,
    {
        self.descend_with(chooser);
        self.descend_to_leaf(chooser);
    }

    fn descend_to_leaf(&mut self, chooser: IdxChooser<'_, K>)
    where
        K: Ord,
    {
        while self.descend_if_possible(chooser) {}
    }

    #[allow(clippy::missing_panics_doc)]
    #[allow(dead_code)]
    pub fn back_inorder(&mut self) -> Option<(&K, &V)>
    where
        K: Ord,
    {
        if self.node.depth > 0 {
            self.descend_to_leaf(IdxChooser::Right);
        } else {
            while self.idx.idx() == 0 {
                if self.ascend().is_none() {
                    self.descend_to_leaf(IdxChooser::Left);
                    return None;
                }
            }

            self.idx = self.idx - 1;
        }

        Some(self.keyval().unwrap())
    }

    #[allow(clippy::missing_panics_doc)]
    #[allow(dead_code)]
    pub fn next_inorder(&mut self) -> Option<(&K, &V)>
    where
        K: Ord,
    {
        self.idx = self.idx + 1;

        let is_internal = self.node.depth > 0;
        while self.idx.idx() >= self.node().entries.len + u8::from(is_internal) {
            if self.ascend().is_none() {
                self.descend_to_leaf(IdxChooser::Right);

                return None;
            }
        }

        if is_internal {
            self.descend_to_leaf(IdxChooser::Left);
        }

        Some(self.keyval().unwrap())
    }

    pub fn find<'a>(&'a mut self, key: &K) -> Option<(&'a K, &'a V)>
    where
        K: Ord,
    {
        self.choose_idx(IdxChooser::Partition(key));

        loop {
            match self.keyval() {
                Some((k, v)) if k == key => return Some((k, v)),
                _ => {}
            }

            if !self.descend_if_possible(IdxChooser::Partition(key)) {
                return None;
            }
        }
    }
}

impl<K, V> Cursor<'_, Mut, K, V> {
    pub const fn node_mut(&mut self) -> &mut UnkownNode<K, V> {
        &mut self.node.node
    }

    pub fn keyval_mut(&mut self) -> Option<(&mut K, &mut V)> {
        let idx = self.idx.idx();
        self.node_mut().entries.get_mut(idx)
    }

    fn swap_remove(&mut self) -> (K, V)
    where
        K: Ord,
    {
        assert!(
            self.node.depth > 0,
            "swap_remove can only be called on internal nodes"
        );
        let swap_kv = unsafe {
            self.node_mut()
                .reborrow_mut()
                .as_internal_mut()
                .entries
                .get_mut(self.idx.idx())
                .unwrap()
        };

        self.descend_to_leaf(IdxChooser::Right);

        let replacement = self.node.node.entries.remove(self.idx.idx()).unwrap();

        (
            core::mem::replace(swap_kv.0, replacement.0),
            core::mem::replace(swap_kv.1, replacement.1),
        )
    }

    fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        if self.find(&key).is_some() {
            return Some(core::mem::replace(self.keyval_mut().unwrap().1, value));
        }

        while self.node().is_full() {
            self.ascend().expect("root should never be full");
        }

        while self.node.depth > 0 {
            let internal = unsafe { self.node.node.as_internal_mut() };
            internal.split_child(self.idx, self.node.depth);

            self.descend_with(IdxChooser::Partition(&key));
        }

        let idx = self.idx.idx();
        self.node_mut()
            .as_leaf_mut()
            .entries
            .insert(idx, (key, value));

        None
    }

    pub fn remove(&mut self) -> (K, V)
    where
        K: Ord,
    {
        let kv = match self.node.depth {
            0 => self.node.node.entries.remove(self.idx.idx()).unwrap(),
            _ => self.swap_remove(),
        };

        while self.node().entries.len < MIN_KEYS && self.ascend().is_some() {
            let internal = unsafe { self.node.node.as_internal_mut() };
            internal.grow_child(self.idx, self.node.depth);
        }

        kv
    }
}

#[derive(Deref, DerefMut)]
struct RootNode<K, V>(UnkownNode<K, V>);

impl<K, V> RootNode<K, V> {
    fn flatten(&mut self, depth: u8) {
        let internal = unsafe { self.as_internal_mut() };
        let mut new_root = internal.entries.take_child(ChildIdx::START);

        if depth == 1 {
            new_root.entries.drain_fully(&mut internal.entries, 0);
            new_root.dealloc(depth - 1);
        } else {
            self.dealloc(depth);
            *self = Self(new_root);
            self.parent = None;
        }
    }

    fn split(&mut self, height: u8)
    where
        K: Ord,
    {
        let (middle, mut right) = match height {
            0 => self.as_leaf_mut().split(),
            _ => unsafe { self.as_internal_mut() }.split(),
        };

        let mut left = if height > 0 {
            core::mem::replace(&mut self.0, InternalNode::default().into())
        } else {
            unsafe {
                core::mem::transmute_prefix::<_, LeafNode<K, V>>(core::mem::take(
                    self.as_internal_mut(),
                ))
                .into()
            }
        };

        let parent = Some(self.0.0.cast::<InternalNode<K, V>>());
        left.parent = parent;
        right.parent = parent;

        let root = unsafe { self.as_internal_mut() };
        root.entries.len = 0;
        root.entries.set_child(ChildIdx::START, left);
        root.entries.insert(ChildIdx::right(0), middle, right);
    }
}

impl<K, V> Default for RootNode<K, V> {
    fn default() -> Self {
        Self(InternalNode::default().into())
    }
}

struct UnkownNode<K, V>(NonNull<LeafNode<K, V>>);

impl<K, V> Clone for UnkownNode<K, V> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

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

impl<K, V> From<&mut LeafNode<K, V>> for UnkownNode<K, V> {
    fn from(node: &mut LeafNode<K, V>) -> Self {
        Self(NonNull::from(node))
    }
}

impl<K, V> From<&mut InternalNode<K, V>> for UnkownNode<K, V> {
    fn from(node: &mut InternalNode<K, V>) -> Self {
        unsafe {
            Self(NonNull::new_unchecked(
                (&raw mut *node).cast::<LeafNode<K, V>>(),
            ))
        }
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

    #[cfg(test)]
    fn assert_invariants(&self, depth: u8)
    where
        K: Ord,
    {
        match depth {
            0 => self.as_leaf().assert_invariants(),
            _ => unsafe { self.as_internal().assert_invariants(depth) },
        }
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
            _ => unsafe { self.as_internal_mut().split_right() },
        }
    }

    const unsafe fn reborrow<'b>(&self) -> &'b Self {
        unsafe { core::mem::transmute(self) }
    }

    const unsafe fn reborrow_mut<'b>(&mut self) -> &'b mut Self {
        unsafe { core::mem::transmute(self) }
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

    fn dealloc(&mut self, depth: u8) {
        unsafe {
            match depth {
                0 => drop(Box::from_non_null(self.0)),
                _ => drop(Box::from(self.as_internal_mut())),
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

    fn display(&self, depth: u8) -> String
    where
        K: Display,
        V: Display,
    {
        match depth {
            0 => self.as_leaf().display(),
            _ => unsafe { self.as_internal().display(depth) },
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
            keys: InlineArray::default(),
            values: InlineArray::default(),
            len: 0,
        }
    }
}

impl<K, V> Entries<K, V> {
    fn drop(&mut self) {
        for i in 0..self.len {
            unsafe_both!(self => [i as usize].assume_init_drop(););
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

    fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        (0..self.len).map(|i| unsafe_both!((self => .get(i as usize),)))
    }

    fn iter_keys(&self) -> impl Iterator<Item = &K> {
        self.keys
            .iter()
            .take(self.len as usize)
            .map(|k| unsafe { k.assume_init_ref() })
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

#[repr(C)]
#[derive(Debug)]
struct LeafNode<K, V> {
    parent: Option<NonNull<InternalNode<K, V>>>,
    entries: Entries<K, V>,
}

impl<K, V> Default for LeafNode<K, V> {
    fn default() -> Self {
        Self {
            parent: None,
            entries: Entries::default(),
        }
    }
}

impl<K, V> LeafNode<K, V> {
    fn split_right(&mut self) -> Self {
        let mut right = Self {
            parent: self.parent,
            ..Default::default()
        };

        self.entries
            .drain(self.entries.len - MIN_KEYS, MIN_KEYS, &mut right.entries, 0);

        right
    }

    const fn is_full(&self) -> bool {
        self.entries.len == MAX_KEYS
    }

    fn partition_point(&self, key: &K) -> u8
    where
        K: Ord,
    {
        self.entries
            .iter_keys()
            .position(|k| k >= key)
            .map_or(self.entries.len, |i| i.try_into().unwrap())
    }

    fn split(&mut self) -> ((K, V), UnkownNode<K, V>)
    where
        K: Ord,
    {
        let right = self.split_right();
        let middle = self.entries.remove(MIN_KEYS).unwrap();

        (middle, right.into())
    }

    fn display(&self) -> String
    where
        K: Display,
        V: Display,
    {
        let mut s = String::from('[');

        self.entries
            .iter()
            .map(|(k, v)| format!("{k}: {v}, "))
            .collect_into(&mut s);

        s.replace_end(2, "]");

        s
    }

    #[cfg(test)]
    fn assert_invariants(&self)
    where
        K: Ord,
    {
        assert!(self.entries.iter_keys().is_sorted(), "keys are not sorted");
        assert!(self.entries.len <= MAX_KEYS, "too many keys in leaf node");
        assert!(
            self.entries.len >= MIN_KEYS || self.parent.is_none(),
            "too few keys in leaf node"
        );
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
            Self::Left => ChildIdx::START,
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

    #[allow(dead_code)]
    fn choose<T>(self, left: T, right: T) -> T {
        match self {
            Self::Left => left,
            Self::Right => right,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChildIdx {
    idx: u8,
    side: Side,
}

impl core::ops::Sub<u8> for ChildIdx {
    type Output = Self;

    fn sub(self, rhs: u8) -> Self::Output {
        Self::new(self.idx - rhs, self.side)
    }
}

impl core::ops::Add<u8> for ChildIdx {
    type Output = Self;

    fn add(self, rhs: u8) -> Self::Output {
        Self::new(self.idx + rhs, self.side)
    }
}

impl ChildIdx {
    const START: Self = Self::left(0);

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

    fn iter_children(&self) -> impl Iterator<Item = &UnkownNode<K, V>> {
        (0..=self.entries.len).map(|i| unsafe { self.children.get(i as usize) })
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
                    .entry_with(|f| self.iter_children().try_for_each(|c| c.fmt(f, depth - 1)))
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
        let child = unsafe {
            self.children
                .shift_left(idx.child_idx() as usize..self.len() as usize + 1, None)
        };

        (child, self.entries.remove(idx.idx()).unwrap())
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
        self.drain(ChildIdx::START, self.len(), other, other_idx);
    }

    fn drain_children(&mut self, idx: ChildIdx, len: u8, other: &mut Self, other_idx: ChildIdx) {
        let old_child_len = other.entries.len - len + 1;
        let len = len + u8::from(other_idx.side.is_left());

        other.children.move_within(
            other_idx.child_idx() as usize..old_child_len as usize,
            (other_idx.child_idx() + len) as usize,
        );

        let range = idx.child_idx() as usize..(idx.child_idx() + len) as usize;
        self.children
            .move_to_other(range, &mut other.children, other_idx.child_idx() as usize);
    }

    #[allow(dead_code)]
    fn children(&self) -> &[UnkownNode<K, V>] {
        unsafe { self.children.get(0..=self.len() as usize) }
    }

    fn children_mut(&mut self) -> &mut [UnkownNode<K, V>] {
        unsafe { self.children.get_mut(0..=self.len() as usize) }
    }
}

#[repr(C)]
struct InternalNode<K, V> {
    parent: Option<NonNull<Self>>,
    entries: ChildEntries<K, V>,
}

impl<K, V> Default for InternalNode<K, V> {
    fn default() -> Self {
        Self {
            parent: None,
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

    fn display(&self, depth: u8) -> String
    where
        K: Display,
        V: Display,
    {
        let mut s = String::from("[");

        self.entries
            .iter()
            .map(|(k, v)| format!("{k}: {v} | "))
            .collect_into(&mut s);

        s.replace_end(3, "]\n");

        self.entries
            .iter_children()
            .enumerate()
            .for_each(|(i, child)| {
                let child = child.display(depth - 1);

                let (start_prefix, prefix) = if i == self.entries.len() as usize {
                    ("└── ", "   ")
                } else {
                    ("├── ", "│  ")
                };

                child.lines().enumerate().for_each(|(i, line)| {
                    let prefix = if i == 0 { start_prefix } else { prefix };

                    writeln!(s, "{prefix}{line}").unwrap();
                });
            });

        s
    }

    fn partition_point(&self, key: &K) -> ChildIdx
    where
        K: Ord,
    {
        ChildIdx::left(self.as_node().partition_point(key))
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

    fn split_right(&mut self) -> UnkownNode<K, V> {
        let mut right_box: UnkownNode<K, V> = Self {
            parent: self.parent,
            entries: ChildEntries {
                entries: self.as_node_mut().split_right().entries,
                children: InlineArray::default(),
            },
        }
        .into();
        let right_parent = Some(NonNull::from(unsafe { right_box.as_internal_mut() }));
        let right = unsafe { right_box.as_internal_mut() };

        self.entries.drain_children(
            ChildIdx::left(MIDDLE),
            MIN_KEYS,
            &mut right.entries,
            ChildIdx::START,
        );

        right
            .entries
            .children_mut()
            .iter_mut()
            .take(MIN_KEYS as usize + 1)
            .for_each(|c| c.parent = right_parent);

        right_box
    }

    fn split_child(&mut self, idx: ChildIdx, depth: u8) {
        let child = self.entries.get_child_mut(idx).unwrap();
        let right = child.split_right(depth - 1);

        let middle = child.entries.remove(MIN_KEYS).unwrap();

        self.entries.insert(idx.next(), middle, right);
    }

    fn drop(&mut self, depth: u8) {
        self.entries.drop();

        self.entries
            .children_mut()
            .iter_mut()
            .for_each(|c| c.drop(depth - 1));
    }

    fn split(&mut self) -> ((K, V), UnkownNode<K, V>)
    where
        K: Ord,
    {
        let right = self.split_right();
        let middle = self.as_node_mut().entries.remove(MIN_KEYS).unwrap();

        (middle, right)
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

            sibling.drain_fully(child, dest);
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

            let (mut c, kv) = sibling.entries.remove(sibling_remove);
            let kv = self.entries.replace(splitting_kv, kv).unwrap();

            let child = unsafe { self.entries.get_child_mut(idx).unwrap().as_internal_mut() };

            c.parent = Some(NonNull::from(&mut *child));

            child.entries.insert(child_add, kv, c);
        }
    }

    #[cfg(test)]
    fn assert_invariants(&self, depth: u8)
    where
        K: Ord,
    {
        self.as_node().assert_invariants();

        let parent = Some(NonNull::from(self));

        self.entries.children().iter().for_each(|c| {
            assert_eq!(c.parent, parent, "child's parent pointer is incorrect");
            c.assert_invariants(depth - 1);
        });

        let mut idx = ChildIdx::START;
        while idx.child_idx() <= self.entries.len() {
            self.assert_child_ranges(idx);
            idx = idx.next();
        }
    }

    #[cfg(test)]
    fn assert_child_ranges(&self, idx: ChildIdx)
    where
        K: Ord,
    {
        let child = self.entries.get_child(idx).unwrap();
        let key = self.entries.get(idx.idx()).unwrap().0;
        let ordering = idx.side.choose(Ordering::Less, Ordering::Greater);

        child
            .entries
            .iter_keys()
            .map(|k| k.cmp(key))
            .for_each(|ord| {
                assert_eq!(ord, ordering, "child's keys are not in the correct range");
            });
    }

    fn drain_fully(&mut self, child: &mut Self, dest: ChildIdx) {
        self.entries.drain_fully(&mut child.entries, dest);

        let parent = Some(NonNull::from(&mut *child));
        child
            .entries
            .children_mut()
            .iter_mut()
            .for_each(|c| c.parent = parent);
    }
}

#[cfg(test)]
mod btree_tests {
    use super::*;
    use crate::rng::Rng;
    use crate::rng::SimpleRng;

    #[test]
    fn test_btree_cursor() {
        let mut btree = Btree::default();

        for i in 0..100 {
            btree.insert(i, i * 10);
        }

        let mut max = btree.max();
        let mut min = btree.min();

        assert_eq!(max.next_inorder(), None);
        assert_eq!(min.back_inorder(), None);

        for i in 1..100 {
            assert_eq!(max.back_inorder(), Some((&(99 - i), &((99 - i) * 10))));
            assert_eq!(min.next_inorder(), Some((&i, &(i * 10))));
        }

        assert_eq!(max.back_inorder(), None);
        assert_eq!(min.next_inorder(), None);
    }

    #[test]
    fn test_btree_cursor_interleaved() {
        let mut btree = Btree::default();

        for i in 0..1000 {
            btree.insert(i, i * 10);
        }

        let mut cursor = btree.min();
        let mut i = 0;
        for _ in 0..10000 {
            if SimpleRng::random() % 2 == 0 {
                i -= 1;

                println!("back_inorder: i = {i}");

                assert_eq!(
                    cursor.back_inorder().map(|(&k, &v)| (k, v)),
                    if i >= 0 { Some((i, i * 10)) } else { None }
                );
            } else {
                i += 1;

                println!("next_inorder: i = {i}");

                assert_eq!(
                    cursor.next_inorder().map(|(&k, &v)| (k, v)),
                    if i < 1000 { Some((i, i * 10)) } else { None }
                );
            }

            i = i.clamp(0, 999);
        }
    }

    #[test]
    fn test_btree_insert_and_get() {
        let mut btree = Btree::default();

        for i in 0..100 {
            btree.insert(i, i * 10);
            btree.assert_invariants();
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
        btree.assert_invariants();
        assert_eq!(btree.get(&1), Some(&10));

        assert_eq!(btree.insert(1, 20), Some(10));
        btree.assert_invariants();
        assert_eq!(btree.get(&1), Some(&20));
    }

    #[test]
    fn test_empty_btree_get() {
        let btree: Btree<u64, u64> = Btree::default();
        btree.assert_invariants();
        assert_eq!(btree.get(&1), None);
    }

    #[test]
    fn test_btree_drop_safety() {
        let mut btree = Btree::default();

        for i in 0..100 {
            btree.insert(i.to_string(), (i * 10).to_string());
            btree.assert_invariants();
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
            btree.assert_invariants();
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
            btree.assert_invariants();
            assert_eq!(btree.get(&key), None);
        }
    }

    #[test]
    fn test_btree_insert_and_get_random() {
        let mut btree = Btree::default();

        let keys: Vec<u64> = (0..1000).map(|_| SimpleRng::random()).collect();

        for &key in &keys {
            btree.insert(key, key);
            btree.assert_invariants();
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

            btree.assert_invariants();
        }
    }
}
