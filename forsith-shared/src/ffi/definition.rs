use crate::{
    ffi::{
        FFIFunction,  FFIPrimitive,
        FFIStruct, FFIType, FFIValueType, Indirection,
    },
    interner::InternedString,
};

impl<'a> FFIType<'a> {
    #[must_use]
    pub fn from_c_value_type(
        s: &'a str,
        indirection: Vec<Indirection>,
    ) -> Self {
        let value_type = FFIValueType::from_c_type(s);
        Self {
            value_type,
            indirection,
        }
    }
}

impl<'a> FFIValueType<'a> {
    #[must_use]
    pub fn from_c_type(s: &'a str) -> Self {
        match s {
            "void" => Self::Void,
            _ if let Some(primitive) = FFIPrimitive::from_c_type(s) => Self::Primitive(primitive),
            _ => Self::Custom(s),
        }
    }
}

impl FFIPrimitive {
    #[must_use]
    pub fn from_c_type(s: &str) -> Option<Self> {
        match s {
            "bool" => Some(Self::Bool),
            "char" => Some(Self::Char),
            "int8_t" => Some(Self::I8),
            "uint8_t" => Some(Self::U8),
            "int16_t" => Some(Self::I16),
            "uint16_t" => Some(Self::U16),
            "int32_t" => Some(Self::I32),
            "uint32_t" => Some(Self::U32),
            "int64_t" => Some(Self::I64),
            "uint64_t" => Some(Self::U64),
            "float" => Some(Self::F32),
            "double" => Some(Self::F64),
            _ => None,
        }
    }
}
