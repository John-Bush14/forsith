use core::{
    fmt::Debug,
    mem::{MaybeUninit, replace},
    ptr::NonNull,
};

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
        let mut new_root =
            unsafe { self.root.assume_internal_mut().children[0].assume_init_read() };

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
            let child =
                unsafe { self.assume_internal_mut().children[len as usize].assume_init_mut() };

            let kv = child.remove_rightmost_keyval(depth - 1);

            if child.entries.len < MIN_KEYS {
                let len = self.entries.len;
                unsafe { self.assume_internal_mut() }.grow_child(len, depth);
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
        root.node.entries.len = 0;
        root.children[0] = MaybeUninit::new(left);
        root.insert_keyvalchild(0, middle.0, middle.1, right, true);
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

struct Entries<K, V> {
    keys: [MaybeUninit<K>; MAX_KEYS as usize],
    values: [MaybeUninit<V>; MAX_KEYS as usize],
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
                self.keys[i as usize].assume_init_drop();
                self.values[i as usize].assume_init_drop();
            }
        }
    }

    const fn get(&self, idx: u8) -> Option<(&K, &V)> {
        if idx >= self.len {
            return None;
        }

        Some((
            unsafe { self.keys[idx as usize].assume_init_ref() },
            unsafe { self.values[idx as usize].assume_init_ref() },
        ))
    }

    const fn get_mut(&mut self, idx: u8) -> Option<(&mut K, &mut V)> {
        if idx >= self.len {
            return None;
        }

        Some((
            unsafe { self.keys[idx as usize].assume_init_mut() },
            unsafe { self.values[idx as usize].assume_init_mut() },
        ))
    }

    fn remove(&mut self, idx: u8) -> Option<(K, V)> {
        if idx >= self.len {
            return None;
        }

        let range = idx as usize..self.len as usize;
        let [key] = self.keys[range.clone()].shift_left([MaybeUninit::uninit(); 1]);
        let [value] = self.values[range].shift_left([MaybeUninit::uninit(); 1]);
        self.len -= 1;

        Some((unsafe { key.assume_init_read() }, unsafe {
            value.assume_init_read()
        }))
    }

    fn iter_keys(&self) -> impl Iterator<Item = &K> {
        self.keys
            .iter()
            .take(self.len as usize)
            .map(|k| unsafe { k.assume_init_ref() })
    }

    const fn get_val(&self, idx: u8) -> Option<&V> {
        if idx >= self.len {
            return None;
        }

        Some(unsafe { self.values[idx as usize].assume_init_ref() })
    }

    fn drain_into(&mut self, idx: u8, len: u8, other: &mut Self, other_idx: u8) {
        assert!(len <= self.len, "drain range out of bounds");

        other.allocate_slice(other_idx, len);

        self.len -= len;

        unsafe {
            self.keys.as_ptr().add(idx as usize).copy_to_nonoverlapping(
                other.keys.as_mut_ptr().add(other_idx as usize),
                len as usize,
            );
            self.values
                .as_ptr()
                .add(idx as usize)
                .copy_to_nonoverlapping(
                    other.values.as_mut_ptr().add(other_idx as usize),
                    len as usize,
                );
        }
    }

    const fn get_val_mut(&mut self, idx: u8) -> Option<&mut V> {
        if idx >= self.len {
            return None;
        }

        unsafe { Some(self.values[idx as usize].assume_init_mut()) }
    }

    fn replace(&mut self, idx: u8, key: K, value: V) -> Option<(K, V)> {
        self.get_mut(idx)
            .map(|(k, v)| (core::mem::replace(k, key), core::mem::replace(v, value)))
    }

    fn insert(&mut self, idx: u8, key: K, value: V) {
        assert!(idx <= self.len, "insert index out of bounds");

        let range = idx as usize..=self.len as usize;
        self.keys[range.clone()].shift_right([MaybeUninit::new(key); 1]);
        self.values[range].shift_right([MaybeUninit::new(value); 1]);
        self.len += 1;
    }

    const fn allocate_slice(&mut self, idx: u8, len: u8) {
        let keys_src = unsafe { self.keys.as_ptr().add(idx as usize) };
        let keys_dst = unsafe { self.keys.as_mut_ptr().add((idx + len) as usize) };
        let values_src = unsafe { self.values.as_ptr().add(idx as usize) };
        let values_dst = unsafe { self.values.as_mut_ptr().add((idx + len) as usize) };

        unsafe {
            keys_src.copy_to(keys_dst, (self.len - idx) as usize);
            values_src.copy_to(values_dst, (self.len - idx) as usize);
        }

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
            .field(
                "keys",
                &self
                    .keys
                    .iter()
                    .take(self.len as usize)
                    .map(|k| unsafe { k.assume_init_ref() })
                    .collect::<Vec<_>>(),
            )
            .field(
                "values",
                &self
                    .values
                    .iter()
                    .take(self.len as usize)
                    .map(|v| unsafe { v.assume_init_ref() })
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

#[derive(Debug)]
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
struct InternalNode<K, V> {
    node: LeafNode<K, V>,
    children: [MaybeUninit<UnkownNode<K, V>>; MAX_KEYS as usize + 1],
}

impl<K, V> Default for InternalNode<K, V> {
    fn default() -> Self {
        Self {
            node: LeafNode::default(),
            children: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }
}

impl<K, V> InternalNode<K, V> {
    fn remove_keyvalchild(&mut self, idx: u8, right: bool) -> (K, V, UnkownNode<K, V>) {
        let child = unsafe { self.children[idx as usize + usize::from(right)].assume_init_read() };
        self.children[idx as usize + usize::from(right)..=self.node.entries.len as usize]
            .shift_left([MaybeUninit::uninit(); 1]);

        let keyval = self.node.entries.remove(idx).unwrap();

        (keyval.0, keyval.1, child)
    }

    fn fmt(&self, depth: u8, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    where
        K: Debug,
        V: Debug,
    {
        f.debug_struct("InternalNode")
            .field("node", &self.node)
            .field_with("children", |f| {
                self.children
                    .iter()
                    .take(self.node.entries.len as usize + 1)
                    .try_for_each(|c| unsafe { c.assume_init_ref().fmt(f, depth - 1) })
            })
            .finish()
    }

    fn get(&self, key: &K, depth: u8) -> Option<&V>
    where
        K: Ord,
    {
        let idx = self.node.partition_point(key);

        if self.node.idx_eq_key(idx, key) {
            return Some(self.node.entries.get_val(idx).unwrap());
        }

        let child = unsafe { self.children[idx as usize].assume_init_ref() };
        child.get(key, depth - 1)
    }

    fn remove(&mut self, key: &K, depth: u8) -> Option<(K, V)>
    where
        K: Ord,
    {
        let idx = self.node.partition_point(key);

        if self.node.idx_eq_key(idx, key) {
            let left_child = unsafe { self.children[idx as usize].assume_init_mut() };
            let (pred_key, pred_value) = left_child.remove_rightmost_keyval(depth - 1);
            let kv = self
                .node
                .entries
                .replace(idx, pred_key, pred_value)
                .unwrap();

            let left_child = unsafe { self.children[idx as usize].assume_init_mut() };
            if left_child.entries.len < MIN_KEYS {
                self.grow_child(idx, depth);
            }

            return Some(kv);
        }
        let kv = unsafe { self.children[idx as usize].assume_init_mut() }.remove(key, depth - 1);
        let child_len = unsafe { self.children[idx as usize].assume_init_ref() }
            .entries
            .len;

        let child_idx = if child_len < MIN_KEYS {
            self.grow_child(idx, depth)
        } else {
            idx
        };

        let child = unsafe { self.children[child_idx as usize].assume_init_mut() };
        assert!(
            child.entries.len >= MIN_KEYS,
            "child must have at least MIN_KEYS after growing"
        );

        kv
    }

    fn grow_child(&mut self, idx: u8, depth: u8) -> u8 {
        let Some((side, Some(sibling))) = self
            .child_siblings(idx)
            .into_iter()
            .enumerate()
            .max_by_key(|(_, s)| s.as_ref().map_or(0, |s| s.entries.len))
        else {
            unreachable!("child should have a sibling");
        };

        let left = side == 0;
        let right = side == 1;
        if sibling.entries.len > MIN_KEYS {
            let sibling_remove = if left { sibling.entries.len - 1 } else { 0 };

            if depth - 1 == 0 {
                let kv = sibling.entries.remove(sibling_remove).unwrap();
                let kv = self
                    .node
                    .entries
                    .replace(idx - u8::from(left), kv.0, kv.1)
                    .unwrap();

                let child = unsafe { self.children[idx as usize].assume_init_mut() };

                let child_add = if left { 0 } else { child.entries.len };
                child.entries.insert(child_add, kv.0, kv.1);
            } else {
                let sibling = unsafe { sibling.assume_internal_mut() };
                let (k, v, c) = sibling.remove_keyvalchild(sibling_remove, left);

                let (k, v) = self
                    .node
                    .entries
                    .replace(idx - u8::from(left), k, v)
                    .unwrap();

                let child = unsafe {
                    self.children[idx as usize]
                        .assume_init_mut()
                        .assume_internal_mut()
                };
                let child_add = if left { 0 } else { child.node.entries.len };
                child.insert_keyvalchild(child_add, k, v, c, right);
            }
            idx
        } else {
            let (k, v, mut sibling) = self.remove_keyvalchild(idx - u8::from(left), right);
            let child =
                unsafe { self.children[idx as usize - usize::from(left)].assume_init_mut() };
            let child_add = if left { 0 } else { child.entries.len };

            if depth - 1 == 0 {
                child.entries.insert(child_add, k, v);

                let len = sibling.entries.len;
                sibling
                    .entries
                    .drain_into(0, len, &mut child.entries, child_add + u8::from(right));
            } else {
                let child = unsafe { child.assume_internal_mut() };
                let sibling = unsafe { sibling.assume_internal_mut() };
                let sibling_len = sibling.node.entries.len;

                let c = unsafe {
                    replace(
                        &mut sibling.children[sibling_len as usize],
                        MaybeUninit::uninit(),
                    )
                    .assume_init_read()
                };
                child.insert_keyvalchild(child_add, k, v, c, right);

                for i in 0..sibling_len {
                    let (k, v, c) = sibling.remove_keyvalchild(0, false);
                    child.insert_keyvalchild(child_add + u8::from(right) + i, k, v, c, false);
                }
            }

            sibling.dealloc(depth - 1);
            idx - u8::from(left)
        }
    }

    fn child_siblings(&mut self, idx: u8) -> [Option<&mut UnkownNode<K, V>>; 2] {
        if idx == 0 {
            [None, Some(unsafe { self.children[1].assume_init_mut() })]
        } else if idx == self.node.entries.len {
            [
                Some(unsafe { self.children[idx as usize - 1].assume_init_mut() }),
                None,
            ]
        } else {
            self.children
                .get_disjoint_mut([idx as usize - 1, idx as usize + 1])
                .map(|[left, right]| {
                    [
                        Some(unsafe { left.assume_init_mut() }),
                        Some(unsafe { right.assume_init_mut() }),
                    ]
                })
                .unwrap()
        }
    }

    fn insert_keyvalchild(
        &mut self,
        idx: u8,
        key: K,
        value: V,
        child: UnkownNode<K, V>,
        right: bool,
    ) {
        self.children[idx as usize + usize::from(right)..=self.node.entries.len as usize + 1]
            .shift_right([MaybeUninit::new(child); 1]);

        self.node.entries.insert(idx, key, value);
    }

    fn split_right(&mut self) -> Self {
        let mut right = Self {
            node: self.node.split_right(),
            children: unsafe { MaybeUninit::uninit().assume_init() },
        };

        for i in 0..=MIN_KEYS as usize {
            right.children[i] = replace(
                &mut self.children[MIDDLE as usize + i],
                MaybeUninit::uninit(),
            );
        }

        right
    }

    fn insert(&mut self, key: K, value: V, depth: u8) -> Option<V>
    where
        K: Ord,
    {
        let mut idx = self.node.partition_point(&key);

        if self.node.idx_eq_key(idx, &key) {
            return Some(core::mem::replace(
                self.node.entries.get_val_mut(idx).unwrap(),
                value,
            ));
        }

        let child = unsafe { self.children[idx as usize].assume_init_mut() };

        if child.entries.len == MAX_KEYS {
            let right = child.split_right(depth - 1);
            let middle = child.entries.remove(MIN_KEYS).unwrap();

            self.insert_keyvalchild(idx, middle.0, middle.1, right, true);
            idx = self.node.partition_point(&key);

            if self.node.idx_eq_key(idx, &key) {
                return Some(core::mem::replace(
                    self.node.entries.get_val_mut(idx).unwrap(),
                    value,
                ));
            }
        }

        let child = unsafe { self.children[idx as usize].assume_init_mut() };
        child.insert(key, value, depth - 1)
    }

    fn drop_children(&mut self, depth: u8) {
        self.node.entries.drop();

        for i in 0..=self.node.entries.len as usize {
            let child = unsafe { self.children[i].assume_init_mut() };
            child.drop(depth - 1);
        }
    }

    fn split_as_root(&mut self) -> (UnkownNode<K, V>, (K, V), UnkownNode<K, V>)
    where
        K: Ord,
    {
        let right = self.split_right();
        let middle = self.node.entries.remove(MIN_KEYS).unwrap();
        let left = core::mem::take(&mut *self);

        (left.into(), middle, right.into())
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
