use core::{mem::{MaybeUninit, replace}, ptr::NonNull, fmt::Debug};

const MAX_KEYS: u8 = 11;
const MIN_KEYS: u8 = MAX_KEYS / 2;
const MIDDLE: u8 = MIN_KEYS + 1;

pub struct Btree<K, V> {
    root: Box<InternalNode<K, V>>,
    depth: u8,
}

impl<K: Debug, V: Debug> Debug for Btree<K, V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Btree")
            .field("depth", &self.depth)
            .field_with("root", |f| match self.depth {
                0 => self.root.node.fmt(f),
                _ => self.root.fmt(self.depth, f),
            })
            .finish()
    }
}

impl<K, V> Btree<K, V> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: Box::new(InternalNode {
                node: LeafNode {
                    parent: None,
                    keys: unsafe { MaybeUninit::uninit().assume_init() },
                    values: unsafe { MaybeUninit::uninit().assume_init() },
                    len: 0,
                },
                children: unsafe { MaybeUninit::uninit().assume_init() },
            }),
            depth: 0,
        }
    }

    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: Ord,
    {
        match self.depth {
            0 => self.root.node.get(key),
            _ => self.root.get(key, self.depth)
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        if self.root.node.len == MAX_KEYS {
            let parent = unsafe {NonNull::new_unchecked(&raw mut *self.root)};

            if self.depth == 0 {
                let mut right = self.root.node.split_right();
                let middle = self.root.node.remove_keyval(MIN_KEYS);
                let mut left = replace(&mut self.root.node, LeafNode {
                    parent: None,
                    keys: unsafe { MaybeUninit::uninit().assume_init() },
                    values: unsafe { MaybeUninit::uninit().assume_init() },
                    len: 0,
                });
                left.parent = Some(parent);
                right.parent = Some(parent);

                self.root.children[0] = MaybeUninit::new(UnkownNode::from_leaf(left));
                self.root.insert_keyvalchild(0, middle.0, middle.1, UnkownNode::from_leaf(right));
            } else {
                let mut right = self.root.split_right();
                let middle = self.root.node.remove_keyval(MIN_KEYS);
                let mut left = replace(&mut *self.root, InternalNode {
                    node: LeafNode {
                        parent: None,
                        keys: unsafe { MaybeUninit::uninit().assume_init() },
                        values: unsafe { MaybeUninit::uninit().assume_init() },
                        len: 0,
                    },
                    children: unsafe { MaybeUninit::uninit().assume_init() },
                });
                left.node.parent = Some(parent);
                right.node.parent = Some(parent);

                self.root.children[0] = MaybeUninit::new(UnkownNode::from_internal(left));
                self.root.insert_keyvalchild(0, middle.0, middle.1, UnkownNode::from_internal(right));
            }

            self.depth += 1;
        }

        match self.depth {
            0 => self.root.node.insert(key, value),
            _ => self.root.insert(key, value, self.depth)
        }
    }
}

impl<K, V> Drop for Btree<K, V> {
    fn drop(&mut self) {
        if self.depth > 0 {
            self.root.drop_children(self.depth);
        } else {
            self.root.node.drop_keyvals();
        }
    }
}

impl<K, V> Default for Btree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

struct UnkownNode<K, V>(NonNull<LeafNode<K, V>>);
impl<K, V> UnkownNode<K, V> {
    const fn as_leaf(&self) -> &LeafNode<K, V> {
        unsafe { self.0.as_ref() }
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
        unsafe {Self(NonNull::new_unchecked(Box::into_raw(Box::new(node)).cast::<LeafNode<K, V>>()))}
    }

    fn drop(&mut self, depth: u8) {
        if depth == 0 {
            self.as_leaf_mut().drop_keyvals();
            unsafe {
                let _ = Box::from_raw(self.0.as_ptr());
            }
        } else { unsafe {
            self.assume_internal_mut().drop_children(depth);
            let _ = Box::from_raw(self.0.as_ptr().cast::<InternalNode<K, V>>());
        } }
    }
}

struct LeafNode<K, V> {
    parent: Option<NonNull<InternalNode<K, V>>>,
    keys: [MaybeUninit<K>; MAX_KEYS as usize],
    values: [MaybeUninit<V>; MAX_KEYS as usize],
    len: u8,
}

impl<K: Debug, V: Debug> Debug for LeafNode<K, V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LeafNode")
            .field("parent", &self.parent)
            .field("len", &self.len)
            .field("keys", &self.keys.iter().take(self.len as usize).map(|k| unsafe { k.assume_init_ref() }).collect::<Vec<_>>())
            .field("values", &self.values.iter().take(self.len as usize).map(|v| unsafe { v.assume_init_ref() }).collect::<Vec<_>>())
            .finish()
    }
}

impl<K, V> LeafNode<K, V> {
    fn idx_eq_key(&self, idx: u8, key: &K) -> bool
    where
        K: Ord,
    {
        idx < self.len && unsafe { self.keys[idx as usize].assume_init_ref() } == key
    }

    fn get(&self, key: &K) -> Option<&V>
    where
        K: Ord,
    {
        self.keys
            .iter()
            .take(self.len as usize)
            .position(|k| unsafe { k.assume_init_ref() } == key)
            .map(|idx| unsafe { self.values[idx].assume_init_ref() })
    }

    fn remove_keyval(&mut self, idx: u8) -> (K, V) {
        let key = unsafe { self.keys[idx as usize].assume_init_read() };
        let value = unsafe { self.values[idx as usize].assume_init_read() };

        let range = idx as usize..self.len as usize;
        self.keys[range.clone()].shift_left([MaybeUninit::uninit(); 1]);
        self.values[range].shift_left([MaybeUninit::uninit(); 1]);

        self.len -= 1;

        (key, value)
    }

    fn split_right(&mut self) -> Self {
        let mut right = Self {
            parent: self.parent,
            keys: unsafe { MaybeUninit::uninit().assume_init() },
            values: unsafe { MaybeUninit::uninit().assume_init() },
            len: MIN_KEYS,
        };

        for i in 0..MIN_KEYS as usize {
            right.values[i] = replace(&mut self.values[MIDDLE as usize + i], MaybeUninit::uninit());
            right.keys[i] = replace(&mut self.keys[MIDDLE as usize + i], MaybeUninit::uninit());
        }

        self.len -= MIN_KEYS;

        right
    }

    fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        let idx = self.partition_point(&key);

        if self.idx_eq_key(idx, &key) {
            let val = unsafe { self.values[idx as usize].assume_init_mut() };
            return Some(core::mem::replace(val, value));
        }

        self.insert_keyval(idx, key, value);

        None
    }

    fn insert_keyval(&mut self, idx: u8, key: K, value: V) {
        let range = idx as usize..=self.len as usize;
        self.keys[range.clone()].shift_right([MaybeUninit::new(key); 1]);
        self.values[range].shift_right([MaybeUninit::new(value); 1]);

        self.len += 1;
    }

    fn partition_point(&self, key: &K) -> u8
    where
        K: Ord,
    {
        let mut i = 0u8;
        while i < self.len && unsafe { self.keys[i as usize].assume_init_ref() } < key {
            i += 1;
        }
        i
    }

    fn drop_keyvals(&mut self) {
        for i in 0..self.len as usize {
            unsafe {
                self.keys[i].assume_init_drop();
                self.values[i].assume_init_drop();
            }
        }
    }
}

#[repr(C)]
struct InternalNode<K, V> {
    node: LeafNode<K, V>,
    children: [MaybeUninit<UnkownNode<K, V>>; MAX_KEYS as usize + 1],
}

impl<K, V> InternalNode<K, V> {
    fn remove_keyvalchild(&mut self, idx: u8) -> (K, V, UnkownNode<K, V>) {
        let keyval = self.node.remove_keyval(idx);
        let child = unsafe { self.children[idx as usize + 1].assume_init_read() };

        (keyval.0, keyval.1, child)
    }

    fn fmt(&self, depth: u8, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    where
        K: Debug,
        V: Debug,
    {
        f.debug_struct("InternalNode")
            .field("node", &self.node)
            .field_with("children", |f| self.children.iter().take(self.node.len as usize + 1).try_for_each(|c| unsafe { c.assume_init_ref().fmt(f, depth - 1) }))
            .finish()
    }

    fn get(&self, key: &K, depth: u8) -> Option<&V>
    where
        K: Ord,
    {
        let idx = self.node.partition_point(key);

        if self.node.idx_eq_key(idx, key) {
            return Some(unsafe { self.node.values[idx as usize].assume_init_ref() });
        }

        let child = unsafe { self.children[idx as usize].assume_init_ref() };
        child.get(key, depth - 1)
    }

    fn insert_keyvalchild(&mut self, idx: u8, key: K, value: V, child: UnkownNode<K, V>) {
        self.node.insert_keyval(idx, key, value);
        self.children[idx as usize + 1..=self.node.len as usize].shift_right([MaybeUninit::new(child); 1]);
    }

    fn add_remove_middle(&mut self, idx: u8, key: K, value: V, child: UnkownNode<K, V>) -> (K, V, UnkownNode<K, V>) {
        if idx == MIN_KEYS {
            (key, value, child)
        } else {
            let middle = self.remove_keyvalchild(MIN_KEYS);
            let middle_child = unsafe { self.children[MIDDLE as usize].assume_init_read() };

            self.insert_keyvalchild(idx, key, value, child);

            (middle.0, middle.1, middle_child)
        }
    }

    fn split_right(&mut self) -> Self {
        let mut right = Self {
            node: self.node.split_right(),
            children: unsafe { MaybeUninit::uninit().assume_init() },
        };

        for i in 0..=MIN_KEYS as usize {
            right.children[i] = replace(&mut self.children[MIDDLE as usize + i], MaybeUninit::uninit());
        }

        right
    }

    #[allow(clippy::needless_pass_by_value)]
    fn split_child(&mut self, mid: (K, V), right: UnkownNode<K, V>)
    where
        K: Ord,
    {
        let idx = self.node.partition_point(&mid.0);

        if self.node.len == MAX_KEYS {
            let Some(mut parent) = self.node.parent else {
                unreachable!()
            };

            let middle = self.add_remove_middle(idx, mid.0, mid.1, right);
            self.split_right();

            unsafe {parent.as_mut()}.split_child(
                (middle.0, middle.1),
                middle.2
            );

            return;
        }

        self.insert_keyvalchild(idx, mid.0, mid.1, right);
    }

    fn insert(&mut self, key: K, value: V, depth: u8) -> Option<V>
    where
        K: Ord,
    {
        let idx = self.node.partition_point(&key);

        if self.node.idx_eq_key(idx, &key) {
            let val = unsafe { self.node.values[idx as usize].assume_init_mut() };
            return Some(core::mem::replace(val, value));
        }

        let child = unsafe { self.children[idx as usize].assume_init_mut() };

        if child.as_leaf().len == MAX_KEYS {
            let right = child.as_leaf_mut().split_right();
            let middle = child.as_leaf_mut().remove_keyval(MIN_KEYS);

            self.split_child(middle, UnkownNode::from_leaf(right));
        }

        let idx = self.node.partition_point(&key);
        let child = unsafe { self.children[idx as usize].assume_init_mut() };

        child.insert(key, value, depth - 1)
    }

    fn drop_children(&mut self, depth: u8) {
        self.node.drop_keyvals();

        for i in 0..=self.node.len as usize {
            let child = unsafe { self.children[i].assume_init_mut() };
            child.drop(depth - 1);
        }
    }
}

#[cfg(test)]
mod btree_tests {
    use crate::rng::SimpleRng;
    use crate::rng::Rng;
    use super::*;

    #[test]
    fn test_btree_insert_and_get() {
        let mut btree = Btree::new();

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
        let mut btree = Btree::new();

        assert_eq!(btree.insert(1, 10), None);
        assert_eq!(btree.get(&1), Some(&10));

        assert_eq!(btree.insert(1, 20), Some(10));
        assert_eq!(btree.get(&1), Some(&20));
    }

    #[test]
    fn test_empty_btree_get() {
        let btree: Btree<u64, u64> = Btree::new();
        assert_eq!(btree.get(&1), None);
    }

    #[test]
    fn test_btree_drop_safety() {
        let mut btree = Btree::new();

        for i in 0..100 {
            btree.insert(i.to_string(), (i*10).to_string());
        }

        for i in 0..100 {
            assert_eq!(btree.get(&i.to_string()), Some(&(i*10).to_string()));
        }

        drop(btree);
    }

    #[test]
    fn test_btree_insert_and_get_random() {
        let mut btree = Btree::new();

        let keys: Vec<u64> = (0..100).map(|_| SimpleRng::random()).collect();

        for &key in &keys {
            btree.insert(key, key * 10);
        }

        for &key in &keys {
            assert_eq!(btree.get(&key), Some(&(key * 10)));
        }
    }
}
