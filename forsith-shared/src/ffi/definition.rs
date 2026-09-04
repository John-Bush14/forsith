use crate::ffi::{FFIPrimitive, FFIValueType};

impl<'a> FFIValueType<'a> {
    #[must_use]
    pub fn from_c_type(s: &'a str) -> Self {
        if s == "void" {
            Self::Void
        } else if let Some(primitive) = FFIPrimitive::from_c_type(s) {
            Self::Primitive(primitive)
        } else {
            Self::Custom(s)
        }
    }
}

impl FFIPrimitive {
    #[must_use]
    pub fn from_c_type(s: &str) -> Option<Self> {
        Some(match s {
            "bool" => Self::Bool,
            "char" => Self::Char,
            "int8_t" => Self::I8,
            "uint8_t" => Self::U8,
            "int16_t" => Self::I16,
            "uint16_t" => Self::U16,
            "int32_t" => Self::I32,
            "uint32_t" => Self::U32,
            "int64_t" => Self::I64,
            "uint64_t" => Self::U64,
            "float" => Self::F32,
            "double" => Self::F64,
            _ => return None,
        })
    }
}


