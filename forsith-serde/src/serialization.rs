pub trait Serializer {
    type Error;
}

pub trait Serialize<T>: Serializer {
    fn serialize(&mut self, value: &T) -> Result<(), Self::Error>;
}

pub trait SeqSerialize {
    type Seq: SeqSerializer;

    fn serialize_seq(&mut self, len: Option<usize>) -> Self::Seq;

    fn serialize_iter<'a, T: 'a, I>(&mut self, iter: I, len: Option<usize>)
    where
        I: IntoIterator<Item = &'a T>,
        Self::Seq: SerializeSeq<T>,
        Self: Sized
    {
        let mut seq = self.serialize_seq(len);
        for item in iter.into_iter() {
            seq.serialize_element(item);
        }
        seq.end();
    }
}

pub trait SeqSerializer {
    fn end(self);
}

pub trait SerializeSeq<T> {
    fn serialize_element(&mut self, value: &T) -> &mut Self;
}


pub trait MapSerialize {
    type Map: MapSerializer;

    fn serialize_map(&mut self, len: Option<usize>) -> Self::Map;

    fn serialize_iter<'a, K: 'a, V: 'a, I>(&mut self, iter: I, len: Option<usize>)
    where
        I: IntoIterator<Item = (&'a K, &'a V)>,
        Self::Map: SerializeMap<K, V>,
        Self: Sized
    {
        let mut map = self.serialize_map(len);
        for (k, v) in iter.into_iter() {
            map.serialize_kv(k, v);
        }
        map.end();
    }
}

pub trait MapSerializer {
    fn end(self);
}

pub trait SerializeMap<K, V>: MapSerializer {
    fn serialize_kv(&mut self, k: &K, v: &V) -> &mut Self;
}
