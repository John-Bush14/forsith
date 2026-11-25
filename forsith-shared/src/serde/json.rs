use super::serialization::{Serialize, Serializer};
use crate::{
    int::Num,
    serde::{
        SerFormat,
        serialization::{
            MapSerialize, MapSerializer, SeqSerialize, SeqSerializer, SerializeMap, SerializeSeq,
        },
    },
};
use std::io::Write;

pub struct JsonSerializer<'a, W: Write + 'a> {
    writer: W,
    phantom: core::marker::PhantomData<&'a ()>,
}

impl<W: Write> JsonSerializer<'_, W> {
    pub const fn new(writer: W) -> Self {
        Self {
            writer,
            phantom: core::marker::PhantomData,
        }
    }
}

pub struct JsonFormat;

impl SerFormat for JsonFormat {
    type Serializer<'a, W: Write + 'a> = JsonSerializer<'a, W>;

    type Error = std::io::Error;

    fn serialize<'a, T: ?Sized, W: Write + 'a>(value: &T, writer: W) -> Result<(), Self::Error>
    where
        Self::Serializer<'a, W>: Serialize<T> + Serializer<Error = std::io::Error>,
    {
        let mut serializer = JsonSerializer::new(writer);
        serializer.serialize(value)
    }
}

impl<W: Write> Serializer for JsonSerializer<'_, W> {
    type Error = std::io::Error;
}

impl<W: Write> Serialize<String> for JsonSerializer<'_, W> {
    fn serialize(&mut self, value: &String) -> Result<(), Self::Error> {
        write!(self.writer, "\"{value}\"")
    }
}

impl<W: Write, N: Num> Serialize<N> for JsonSerializer<'_, W>
where
    u64: From<N>,
{
    fn serialize(&mut self, value: &N) -> Result<(), Self::Error> {
        write!(self.writer, "{value}")
    }
}

impl<W: Write> Serialize<bool> for JsonSerializer<'_, W> {
    fn serialize(&mut self, value: &bool) -> Result<(), Self::Error> {
        write!(self.writer, "{value}")
    }
}

impl<W: Write> Serialize<()> for JsonSerializer<'_, W> {
    fn serialize(&mut self, _value: &()) -> Result<(), Self::Error> {
        write!(self.writer, "null")
    }
}

impl<W: Write, T> Serialize<Option<T>> for JsonSerializer<'_, W>
where
    Self: Serialize<T>,
{
    fn serialize(&mut self, value: &Option<T>) -> Result<(), Self::Error> {
        match value {
            Some(v) => self.serialize(v),
            None => <Self as Serialize<()>>::serialize(self, &()),
        }
    }
}

impl<'a, W: Write + 'a, T> SeqSerialize<T> for JsonSerializer<'a, W>
where
    JsonSerializer<'a, W>: Serialize<T> + Serializer<Error = std::io::Error>,
{
    type Seq<'b>
        = &'b mut Self
    where
        Self: 'b;

    fn serialize_seq(&mut self, _len: Option<usize>) -> Result<Self::Seq<'_>, Self::Error> {
        write!(self.writer, "[").map(|()| self)
    }
}

impl<W: Write> SeqSerializer for &mut JsonSerializer<'_, W> {
    type Error = std::io::Error;

    fn end(self) {
        write!(self.writer, "]").unwrap();
    }
}

impl<'a, W: Write, T> SerializeSeq<T> for &mut JsonSerializer<'a, W>
where
    JsonSerializer<'a, W>: Serialize<T> + Serializer<Error = std::io::Error>,
{
    fn serialize_element(&mut self, value: &T) -> Result<&mut Self, Self::Error> {
        self.serialize(value)?;
        write!(self.writer, ",")?;

        Ok(self)
    }
}

impl<W: Write> MapSerialize for JsonSerializer<'_, W>
where
    Self: Serializer<Error = std::io::Error>,
{
    type Map<'b>
        = &'b mut Self
    where
        Self: 'b;

    fn serialize_map(&mut self, _len: Option<usize>) -> Result<Self::Map<'_>, Self::Error> {
        write!(self.writer, "{{").map(|()| self)
    }
}

impl<'a, W: Write> MapSerializer for &mut JsonSerializer<'a, W>
where
    JsonSerializer<'a, W>: Serializer<Error = std::io::Error>,
{
    type Error = std::io::Error;

    fn end(self) {
        write!(self.writer, "}}").unwrap();
    }
}

impl<W: Write, K, V> SerializeMap<K, V> for &mut JsonSerializer<'_, W>
where
    Self: Serializer<Error = std::io::Error> + Serialize<K> + Serialize<V>,
{
    fn serialize_kv(&mut self, key: &K, value: &V) -> Result<&mut Self, Self::Error> {
        self.serialize(key)?;
        write!(self.writer, ":")?;
        self.serialize(value)?;
        write!(self.writer, ",")?;

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialize_tests;

    serialize_tests! {
        JsonFormat {
            test_serialize_string: &"Hello, world!".to_string() => b"\"Hello, world!\"".to_vec(),
            test_serialize_u64: &42u64 => b"42".to_vec(),
            test_serialize_bool_true: &true => b"true".to_vec(),
            test_serialize_bool_false: &false => b"false".to_vec(),
            test_serialize_unit: &() => b"null".to_vec(),
            test_serialize_option_some: &Some(42u64) => b"42".to_vec(),
            test_serialize_option_none: &None::<u64> => b"null".to_vec(),
            test_serialize_seq: vec![1u64, 2, 3].as_slice() => b"[1,2,3,]".to_vec(),
        }
    }
}
