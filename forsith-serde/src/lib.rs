pub trait Serializer<T> {
    fn serialize(&mut self, value: &T);
}

pub trait SeqSerializer {
    type Seq: SerializeSeq;

    fn serialize_seq(&mut self, len: Option<usize>) -> Self::Seq;

    fn serialize_iter<'a, T: 'a, I>(&mut self, iter: I, len: Option<usize>)
    where
        I: IntoIterator<Item = &'a T>,
        Self::Seq: Serializer<T>,
        Self: Sized
    {
        let mut seq = self.serialize_seq(len);
        for item in iter.into_iter() {
            seq.serialize_element(item);
        }
        seq.end();
    }
}

pub trait SerializeSeq {
    fn serialize_element<T>(&mut self, value: &T) -> &mut Self
    where
        Self: Serializer<T>;

    fn end(self);
}


pub trait MapSerializer {
    type Map: SerializeMap;

    fn serialize_map(&mut self, len: Option<usize>) -> Self::Map;

    fn serialize_iter<'a, K: 'a, V: 'a, I>(&mut self, iter: I, len: Option<usize>)
    where
        I: IntoIterator<Item = (&'a K, &'a V)>,
        Self::Map: Serializer<K> + Serializer<V>,
        Self: Sized
    {
        let mut map = self.serialize_map(len);
        for (k, v) in iter.into_iter() {
            map.serialize_kv(k, v);
        }
        map.end();
    }
}

pub trait SerializeMap {
    fn serialize_kv<K, V>(&mut self, k: &K, v: &V) -> &mut Self
    where
        Self: Serializer<K> + Serializer<V>;

    fn end(self);
}
