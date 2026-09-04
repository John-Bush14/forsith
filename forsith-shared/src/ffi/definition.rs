use crate::ffi::{FFIField, FFIFieldMetadata, FFIFunction, FFIItem, FFIItems, FFILib, FFILibConfig, FFIMod, FFIPrimitive, FFIStruct, FFIType, FFIValueType, Indirection};


impl<'a> FFILib<'a> {
    pub fn def_struct(&mut self, s: FFIStruct<'a>) {
        self.items.0.push(FFIItem::Struct(s));
    }

    pub fn def_function(&mut self, f: FFIFunction<'a>) {
        self.items.0.push(FFIItem::Function(f));
    }

    pub fn def_mod<R>(&mut self, name: &'a str, def: impl FnOnce(&mut FFIItems<'a>) -> R) -> R {
        let mut items = FFIItems::default();

        let r = def(&mut items);

        self.items.0.push(FFIItem::Mod(FFIMod {
            name,
            items,
        }));

        r
    }
}


impl<'a> FFIField<'a> {
    #[must_use]
    pub fn new(ty: FFIType<'a>, meta: FFIFieldMetadata, optional: bool) -> FFIField<'a> {
        Self {
            ty,
            meta,
            optional,
        }
    }
}

impl<'a> FFILib<'a> {
    #[must_use]
    pub fn new(config: FFILibConfig) -> Self {
        Self {
            items: FFIItems::default(),
            config,
        }
    }
}

impl<'a> FFIStruct<'a> {
    #[must_use]
    pub fn new(name: &'a str, fields: Vec<FFIField<'a>>) -> Self {
        Self { name, fields }
    }
}

impl<'a> FFIFunction<'a> {
    #[must_use]
    pub fn new(name: &'a str, params: Vec<FFIField<'a>>, ret_ty: FFIType<'a>) -> Self {
        Self {
            name,
            params,
            ret_ty,
        }
    }
}

impl<'a> FFIType<'a> {
    #[must_use]
    pub fn new(value_type: FFIValueType<'a>, indirection: Indirection) -> Self {
        Self {
            value_type,
            indirection,
        }
    }

    #[must_use]
    pub fn from_c_value_type(s: &'a str, indirection: Indirection) -> Self {
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


