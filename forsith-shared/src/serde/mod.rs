use std::io::Write;

use crate::serde::serialization::{Serialize, Serializer};

pub mod serialization;

pub mod json;

pub trait SerFormat {
    type Serializer<'a, W: Write + 'a>: Serializer<Error = Self::Error>;

    type Error;

    fn serialize<'a, T: ?Sized, W: Write + 'a>(value: &T, writer: W) -> Result<(), Self::Error>
    where
        Self::Serializer<'a, W>: Serialize<T> + Serializer<Error = Self::Error>;

    fn serialize_to_vec<'a, T: ?Sized>(value: &T) -> Result<Vec<u8>, Self::Error>
    where
        for<'b> Self::Serializer<'b, &'b mut Vec<u8>>: Serialize<T>,
    {
        let mut vec = Vec::new();
        Self::serialize(value, &mut vec)?;
        Ok(vec)
    }
}

#[macro_export]
macro_rules! serialize_tests {
    ($format:ty {
        $($name:ident: $value:expr => $expected:expr,)*
    }) => {
        $(
            #[test]
            fn $name() {
                let expected = $expected;
                let serialized = <$format>::serialize_to_vec($value).unwrap();
                assert_eq!(serialized, expected);
            }
        )*
    };
}
