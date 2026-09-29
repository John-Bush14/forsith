use crate::ffi::{
    FFIField, FFIFuncDef, FFIGenerator, FFIPrimitive, FFIType, FFIValueType, FFiFieldIndirection,
};
use forsith_base::{
    proc_macro::{
        Ident, TokenStream,
        items::{FunctionDefinition, GenericsDefinition, TypeAliasDefinition, Visibility},
    },
    quote,
};

impl FFIType<'_> {
    #[must_use]
    pub fn raw_type(&self) -> TokenStream {
        let mut ty = quote!((@ self.value_type.ident()));

        for indirection in self.indirection.iter().rev() {
            ty = indirection.wrap_type_ptr(ty);
        }

        ty
    }
}

impl FFIPrimitive {
    #[must_use]
    const fn as_str(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Char => "char",
            Self::I8 => "i8",
            Self::U8 => "u8",
            Self::I16 => "i16",
            Self::U16 => "u16",
            Self::I32 => "i32",
            Self::U32 => "u32",
            Self::I64 => "i64",
            Self::U64 => "u64",
            Self::F32 => "f32",
            Self::F64 => "f64",
        }
    }
}

impl core::fmt::Display for FFIValueType<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FFIValueType<'_> {
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            FFIValueType::Void => "()",
            FFIValueType::Custom(s) => s,
            FFIValueType::Primitive(primitive) => primitive.as_str(),
        }
    }

    #[must_use]
    pub fn ident(&self) -> Ident {
        Ident::new(self.as_str())
    }
}

impl FFiFieldIndirection<'_> {
    #[must_use]
    pub fn wrap_type_raw(&self, ty: TokenStream) -> TokenStream {
        if self.mutable {
            quote! { *mut (@ ty) }
        } else {
            quote! { *const (@ ty) }
        }
    }
}

impl FFIField<'_> {
    #[must_use]
    pub fn raw_type(&self) -> TokenStream {
        let mut ty = quote!((@ self.ty.ident()));

        for indirection in self.meta.iter().rev() {
            ty = indirection.wrap_type_raw(ty);
        }

        ty
    }

    #[must_use]
    pub fn raw_definition(&self) -> (Ident, TokenStream) {
        (self.name.to_string().into(), self.raw_type())
    }
}

pub struct FFIFuncGen<'a, 'b> {
    pub generator: &'a FFIGenerator,
    pub func_def: FFIFuncDef<'b>,
}

impl FFIFuncGen<'_, '_> {
    fn raw_param_definitions(&self) -> Vec<(Ident, TokenStream)> {
        self.func_def
            .params
            .iter()
            .map(|param| param.raw_definition())
            .collect()
    }

    #[must_use]
    pub fn raw_fn_definition(&self) -> FunctionDefinition {
        let name = (self.generator.func_map.raw)(self.func_def.name);

        FunctionDefinition {
            attributes: Vec::default(),
            visibility: Visibility::Public,
            constness: false,
            unsafety: true,
            asyncness: false,
            name: name.to_string().into(),
            self_param: None,
            generics: GenericsDefinition::default(),
            params: self.raw_param_definitions(),
            ret_ty: self.func_def.ret_ty.as_ref().map(FFIType::raw_type),
            body: None,
        }
    }

    #[must_use]
    pub fn type_definition(&self) -> TypeAliasDefinition {
        let name = (self.generator.func_map.ty)(self.func_def.name);

        TypeAliasDefinition {
            name: name.to_string().into(),
            generics: GenericsDefinition::default(),
            attributes: vec![],
            visibility: Visibility::Public,
            ty: self.raw_fn_definition().signature(),
        }
    }
}
