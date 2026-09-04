use crate::{ffi::{Constant, FFIFunction, FFIItem, FFIItems, FFILib, FFILibConfig, FFIMod, FFIPrimitive, FFIStruct, FFIType, FFIValueType, Indirection, TypeAlias, UseItem}, interner::{InternedString, StringInterner}};

impl FFILib {
    pub fn def_struct(&mut self, r#struct: FFIStruct) {
        self.items.0.push(FFIItem::Struct(r#struct));
    }

    pub fn def_function(&mut self, function: FFIFunction) {
        self.items.0.push(FFIItem::Function(function));
    }

    pub fn def_mod<R>(&mut self, name: InternedString, def: impl FnOnce(&mut FFIItems) -> R) -> R {
        let mut items = FFIItems::default();

        let r = def(&mut items);

        self.items.0.push(FFIItem::Mod(FFIMod {
            name,
            items,
        }));

        r
    }

    pub fn def_use(&mut self, use_item: UseItem) {
        self.items.0.push(FFIItem::Use(use_item));
    }

    pub fn def_type_alias(&mut self, type_alias: TypeAlias) {
        self.items.0.push(FFIItem::TypeAlias(type_alias));
    }

    pub fn def_constant(&mut self, constant: Constant) {
        self.items.0.push(FFIItem::Constant(constant));
    }

    #[must_use]
    pub fn intern_table(&self, interner: &mut StringInterner) -> InternedFFISymbols {
        InternedFFISymbols {
            void: interner.interned("void"),
            bool: interner.interned("bool"),
            char: interner.interned("char"),
            i8: interner.interned("int8_t"),
            u8: interner.interned("uint8_t"),
            i16: interner.interned("int16_t"),
            u16: interner.interned("uint16_t"),
            i32: interner.interned("int32_t"),
            u32: interner.interned("uint32_t"),
            i64: interner.interned("int64_t"),
            u64: interner.interned("uint64_t"),
            f32: interner.interned("float"),
            f64: interner.interned("double"),
        }
    }
}

pub struct InternedFFISymbols {
    void: InternedString,
    bool: InternedString,
    char: InternedString,
    i8: InternedString,
    u8: InternedString,
    i16: InternedString,
    u16: InternedString,
    i32: InternedString,
    u32: InternedString,
    i64: InternedString,
    u64: InternedString,
    f32: InternedString,
    f64: InternedString,
}

impl FFILib  {
    #[must_use]
    pub fn new(config: FFILibConfig) -> Self {
        Self {
            items: FFIItems::default(),
            config,
        }
    }
}

impl FFIType {
    #[must_use]
    pub fn from_c_value_type(s: InternedString, indirection: Indirection, table: &InternedFFISymbols) -> Self {
        let value_type = FFIValueType::from_c_type(s, table);
        Self {
            value_type,
            indirection,
        }
    }
}

impl FFIValueType {
    #[must_use]
    pub fn from_c_type(s: InternedString, table: &InternedFFISymbols) -> Self {
        if s == table.void {
            Self::Void
        } else if let Some(primitive) = FFIPrimitive::from_c_type(s, table) {
            Self::Primitive(primitive)
        } else {
            Self::Custom(s)
        }
    }
}

impl FFIPrimitive {
    #[must_use]
    pub fn from_c_type(s: InternedString, table: &InternedFFISymbols) -> Option<Self> {
        if s == table.bool {
            Some(Self::Bool)
        } else if s == table.char {
            Some(Self::Char)
        } else if s == table.i8 {
            Some(Self::I8)
        } else if s == table.u8 {
            Some(Self::U8)
        } else if s == table.i16 {
            Some(Self::I16)
        } else if s == table.u16 {
            Some(Self::U16)
        } else if s == table.i32 {
            Some(Self::I32)
        } else if s == table.u32 {
            Some(Self::U32)
        } else if s == table.i64 {
            Some(Self::I64)
        } else if s == table.u64 {
            Some(Self::U64)
        } else if s == table.f32 {
            Some(Self::F32)
        } else if s == table.f64 {
            Some(Self::F64)
        } else {
            None
        }
    }
}


