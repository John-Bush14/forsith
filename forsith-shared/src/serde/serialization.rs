pub trait Serializer {
    type Error;
}

pub trait Serialize<T: ?Sized>: Serializer {
    fn serialize(&mut self, value: &T) -> Result<(), Self::Error>;
}

pub trait SeqSerialize<T: ?Sized>: Serializer {
    type Seq<'a>: SeqSerializer<Error = Self::Error> + SerializeSeq<T>
    where
        Self: 'a;

    fn serialize_seq(&mut self, len: Option<usize>) -> Result<Self::Seq<'_>, Self::Error>;

    fn serialize_iter<'b, 'a, I>(
        &'b mut self,
        iter: I,
        len: Option<usize>,
    ) -> Result<(), Self::Error>
    where
        T: 'a,
        I: IntoIterator<Item = &'a T>,
        Self: Sized,
    {
        let mut seq = self.serialize_seq(len)?;
        for item in iter {
            seq.serialize_element(item)?;
        }
        seq.end();
        Ok(())
    }
}

pub trait SeqSerializer {
    type Error;

    fn end(self);
}

pub trait SerializeSeq<T: ?Sized>: SeqSerializer {
    fn serialize_element(&mut self, value: &T) -> Result<&mut Self, Self::Error>;
}

pub trait MapSerialize: Serializer {
    type Map<'a>: MapSerializer<Error = Self::Error>
    where
        Self: 'a;

    fn serialize_map(&mut self, len: Option<usize>) -> Result<Self::Map<'_>, Self::Error>;

    fn serialize_iter<'b, 'a, K: 'a, V: 'a, I>(
        &'b mut self,
        iter: I,
        len: Option<usize>,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = (&'a K, &'a V)>,
        Self::Map<'b>: SerializeMap<K, V>,
        Self: Sized,
    {
        let mut map = self.serialize_map(len)?;
        for (k, v) in iter {
            map.serialize_kv(k, v)?;
        }
        map.end();
        Ok(())
    }
}

pub trait MapSerializer {
    type Error;

    fn end(self);
}

pub trait SerializeMap<K: ?Sized, V: ?Sized>: MapSerializer {
    fn serialize_kv(&mut self, k: &K, v: &V) -> Result<&mut Self, Self::Error>;
}

impl<S, T> Serialize<[T]> for S
where
    S: SeqSerialize<T>,
{
    fn serialize(&mut self, slice: &[T]) -> Result<(), Self::Error> {
        self.serialize_iter(slice, Some(slice.len()))
    }
}

impl<S, T> Serialize<Vec<T>> for S
where
    S: Serialize<[T]>,
{
    fn serialize(&mut self, vec: &Vec<T>) -> Result<(), Self::Error> {
        self.serialize(vec.as_slice())
    }
}
