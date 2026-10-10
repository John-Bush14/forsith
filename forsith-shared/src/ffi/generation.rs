use crate::ffi::{
    FFIField, FFIFuncDef, FFIGenerator, FFIPrimitive, FFIType, FFIValueType, FFiFieldIndirection,
    IndirectionLen,
};
use forsith_base::{
    proc_macro::{
        Ident, Literal, TokenStream,
        items::{
            FunctionDefinition, FunctionParam, GenericsDefinition, TypeAliasDefinition, Visibility,
        },
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

    #[must_use]
    pub fn wrapped_type(&self) -> TokenStream {
        let mut ty = quote!((@ self.value_type.ident()));

        for indirection in self.indirection.iter().rev() {
            ty = indirection.wrap_type_ref(ty);
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

    /// Wraps an expr with this indirection, for example wraps 'expr' with 'Some(&mut expr)' if
    /// this indirection is optional and mutable.
    #[must_use]
    pub fn wrap_expr_safe(&self, mut expr: TokenStream) -> TokenStream {
        assert!(
            self.len == IndirectionLen::None,
            "Indirection with length is not supported for wrapping expressions"
        );

        if self.mutable {
            expr = quote! {&mut (@ expr)}
        } else {
            expr = quote! {& (@ expr)}
        }

        if self.optional {
            expr = quote! {Some((@ expr))};
        }

        expr
    }

    #[must_use]
    pub fn unwrap_expr_safe(&self, expr: TokenStream) -> TokenStream {
        let unwrap = |expr| match self.len {
            IndirectionLen::None => {
                if self.mutable {
                    quote! {core::ptr::from_mut((@ expr))}
                } else {
                    quote! {core::ptr::from_ref((@ expr))}
                }
            }
            _ => {
                if self.mutable {
                    quote! {(@ expr).as_mut_ptr()}
                } else {
                    quote! {(@ expr).as_ptr()}
                }
            }
        };

        if self.optional {
            let null = if self.mutable {
                quote! {core::ptr::null_mut()}
            } else {
                quote! {core::ptr::null()}
            };

            let as_expr = if self.mutable {
                quote! {as_mut()}
            } else {
                quote! {as_ref()}
            };

            quote! {(@ expr).(@ as_expr).map_or((@ null), |expr| (@ unwrap(quote! {expr})))}
        } else {
            unwrap(expr)
        }
    }

    #[must_use]
    pub fn wrap_type_safe(&self, mut ty: TokenStream, use_vec: bool) -> TokenStream {
        ty = match self.len {
            IndirectionLen::None => ty,
            IndirectionLen::Fixed(len) => quote! { [(@ ty); (@ Literal::Integer(len, None))] },
            IndirectionLen::Field(_) if use_vec => quote! { Vec<(@ ty)> },
            IndirectionLen::Field(_) => quote! { [(@ ty)] },
        };

        ty = if self.mutable {
            quote! {&mut (@ ty)}
        } else {
            quote! {& (@ ty)}
        };

        if self.optional {
            quote! {Option<(@ ty)>}
        } else {
            ty
        }
    }
}

impl FFIField<'_> {
    #[must_use]
    pub fn wrapped_type(&self, use_vec: bool) -> TokenStream {
        let mut ty = quote!((@ self.ty.ident()));

        for indirection in self.meta.iter().rev() {
            ty = indirection.wrap_type_safe(ty, use_vec);
        }

        ty
    }

    #[must_use]
    pub fn raw_type(&self) -> TokenStream {
        let mut ty = quote!((@ self.ty.ident()));

        for indirection in self.meta.iter().rev() {
            ty = indirection.wrap_type_raw(ty);
        }

        ty
    }

    #[must_use]
    pub fn unwrap_expr_safe(&self, mut expr: TokenStream) -> TokenStream {
        for indirection in self.meta.iter().rev() {
            expr = indirection.unwrap_expr_safe(expr);
        }

        expr
    }

    #[must_use]
    pub fn wrap_expr_safe(&self, mut expr: TokenStream) -> TokenStream {
        for indirection in self.meta.iter().rev() {
            expr = indirection.wrap_expr_safe(expr);
        }

        expr
    }

    #[must_use]
    pub fn wrapped_definition(&self, use_vec: bool) -> FunctionParam {
        FunctionParam {
            pattern: quote! {(@ Ident::new(self.name))},
            ty: self.wrapped_type(use_vec),
        }
    }

    #[must_use]
    pub fn raw_definition(&self) -> FunctionParam {
        FunctionParam {
            pattern: quote! {(@ Ident::new(self.name))},
            ty: self.raw_type(),
        }
    }
}

pub struct FFIFuncGen<'a, 'b> {
    pub generator: &'a FFIGenerator,
    pub func_def: FFIFuncDef<'b>,
}

impl FFIFuncGen<'_, '_> {
    fn raw_param_definitions(&self) -> Vec<FunctionParam> {
        self.func_def
            .params
            .iter()
            .map(FFIField::raw_definition)
            .collect()
    }

    fn wrapper_params(&self, len_fields: &[&str], vec_fields: &[&str]) -> Vec<FunctionParam> {
        self.func_def
            .params
            .iter()
            .filter(|arg| !len_fields.contains(&arg.name))
            .map(|arg| FFIField::wrapped_definition(arg, vec_fields.contains(&arg.name)))
            .collect()
    }

    fn get_len_field(&self, field: &str) -> Option<&FFIField<'_>> {
        self.func_def.params.iter().find(|arg| arg.name == field)
    }

    fn unwrapped_raw_fn_expr(&self, raw_fn_expr: TokenStream) -> TokenStream {
        let unwrapped_arg_exprs = self
            .func_def
            .params
            .iter()
            .map(|arg| quote! {(@ arg.unwrap_expr_safe(quote! {(@ Ident::new(arg.name))})),})
            .collect::<TokenStream>();

        quote! {
            (@ raw_fn_expr)((@ unwrapped_arg_exprs))
        }
    }

    fn len_field_params(&self) -> impl Iterator<Item = &FFIField<'_>> {
        self.func_def.params.iter().filter(|arg| {
            arg.meta
                .iter()
                .any(|meta| matches!(meta.len, IndirectionLen::Field(_)))
        })
    }

    fn handle_len_field_params_wrapper(
        &self,
        raw_fn_expr: &TokenStream,
        body: &mut TokenStream,
    ) -> (Vec<&str>, Vec<&str>) {
        let mut len_fields = Vec::new();
        let mut vec_fields = Vec::new();

        for param in self.len_field_params() {
            let IndirectionLen::Field(field) = param.meta[0].len else {
                unreachable!()
            };

            len_fields.push(field);

            let Some(len_field) = self.get_len_field(field) else {
                panic!(
                    "Length field '{}' not found for parameter '{}'",
                    field, param.name
                );
            };

            assert!(
                len_field.meta.len() <= 1,
                "Length field '{}' for parameter '{}' has more then 1 level of indirection, which is not supported",
                field,
                param.name
            );

            let len_field_ident = Ident::new(field);
            let param_ident = Ident::new(param.name);

            match len_field.meta.first() {
                Some(FFiFieldIndirection { optional, .. })
                    if *optional != param.meta[0].optional =>
                {
                    panic!(
                        "Length field '{}' for parameter '{}' has different optionality than the parameter",
                        len_field_ident, param.name
                    );
                }
                Some(FFiFieldIndirection {
                    len: IndirectionLen::Field(_) | IndirectionLen::Fixed(_),
                    ..
                }) => {
                    panic!(
                        "Length field '{}' for parameter '{}' has length field itself?",
                        len_field_ident, param.name
                    );
                }
                None | Some(FFiFieldIndirection { mutable: false, .. }) => {
                    let wrapped_len_field =
                        len_field.wrap_expr_safe(quote! {(@ param_ident).len()});

                    body.extend(quote! {
                        let (@ len_field_ident) = (@ wrapped_len_field);
                    });
                }
                Some(FFiFieldIndirection {
                    mutable: true,
                    optional,
                    ..
                }) => {
                    vec_fields.push(param.name);

                    let raw_len_field = Ident::from(format!("raw_{len_field_ident}"));

                    let mut wrapped_len_field =
                        len_field.wrap_expr_safe(quote! {(@ raw_len_field.clone())});
                    if *optional {
                        wrapped_len_field = quote! {if (@ param_ident.clone()).is_some() {(@ wrapped_len_field)} else {None}};
                    }

                    let condition = if *optional {
                        quote! {if (@ param_ident.clone()).is_some()}
                    } else {
                        quote! {true}
                    };

                    let unwrap = if *optional {
                        quote! {.as_mut().unwrap()}
                    } else {
                        quote! {}
                    };

                    body.extend(quote! {
                        let mut (@ raw_len_field) = 0;
                        let mut (@ len_field_ident.clone()) = (@ wrapped_len_field);

                        (@ condition) {
                            (@ self.unwrapped_raw_fn_expr(raw_fn_expr.clone()));

                            unsafe {(@ param_ident)(@ unwrap.clone()).set_len(**(@ len_field_ident)(@ unwrap) as usize);}
                        }
                    });
                }
            }
        }

        (len_fields, vec_fields)
    }

    /// # Panics
    /// Panics if a length field is not found for a parameter that requires one.
    #[must_use]
    pub fn wrapper_fn_definition(&self, raw_fn_expr: TokenStream) -> FunctionDefinition {
        let name = (self.generator.func_map.wrapper)(self.func_def.name);

        let mut body = TokenStream::new();

        let (len_fields, vec_fields) =
            self.handle_len_field_params_wrapper(&raw_fn_expr, &mut body);

        body.extend(quote! {
            (@ self.unwrapped_raw_fn_expr(raw_fn_expr));
        });

        let mut params = self.wrapper_params(&len_fields, &vec_fields);
        for param in &mut params {
            param.pattern = quote! {mut (@ param.pattern.clone())};
        }

        FunctionDefinition {
            attributes: Vec::default(),
            visibility: Visibility::Public,
            constness: false,
            unsafety: false,
            asyncness: false,
            name: name.to_string().into(),
            self_param: None,
            generics: GenericsDefinition::default(),
            params,
            ret_ty: self.func_def.ret_ty.as_ref().map(FFIType::wrapped_type),
            body: Some(body),
        }
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

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;

    use crate::ffi::FFIStructMap;

    use super::*;

    #[test]
    fn ffi_func_all() {
        let generator = FFIGenerator {
            func_map: crate::ffi::FFiFuncMap {
                loader: |l| Cow::Borrowed(l),
                wrapper: |_| Cow::Owned("wrapped".to_owned()),
                ty: |_| Cow::Owned("ty".to_owned()),
                raw: |_| Cow::Owned("raw".to_owned()),
            },
            struct_map: FFIStructMap {
                raw: |_| "raw".into(),
                wrapper: |_| "wrapper".into(),
            },
        };

        let test_func = FFIFuncDef {
            name: "test",
            params: vec![
                FFIField {
                    name: "test1",
                    ty: FFIValueType::Void,
                    meta: vec![FFiFieldIndirection {
                        mutable: true,
                        optional: true,
                        len: IndirectionLen::Field("test2"),
                    }],
                },
                FFIField {
                    name: "test2",
                    ty: FFIValueType::Primitive(FFIPrimitive::U32),
                    meta: vec![FFiFieldIndirection {
                        mutable: true,
                        optional: true,
                        len: IndirectionLen::None,
                    }],
                },
                FFIField {
                    name: "test3",
                    ty: FFIValueType::Primitive(FFIPrimitive::U32),
                    meta: vec![FFiFieldIndirection {
                        mutable: false,
                        optional: false,
                        len: IndirectionLen::Fixed(5),
                    }],
                },
                FFIField {
                    name: "test4",
                    ty: FFIValueType::Primitive(FFIPrimitive::U32),
                    meta: vec![],
                },
            ],
            ret_ty: Some(FFIType {
                value_type: FFIValueType::Void,
                indirection: Vec::new(),
            }),
        };

        let fgen = FFIFuncGen {
            generator: &generator,
            func_def: test_func,
        };

        println!(
            "raw {} \n wrapper {}",
            fgen.raw_fn_definition().definition(),
            fgen.wrapper_fn_definition(quote! {test}).definition()
        );

        panic!()
    }
}
