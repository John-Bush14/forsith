use crate::{
    proc_macro::{
        TokenStream,
        items::{FunctionDefinition, Indirection, ItemDefinition, TypeAliasDefinition, Visibility},
    },
    quote,
};

impl Visibility {
    /// produces the complete visibility definition for an item, e.g. `pub` or `pub(crate)`.
    #[must_use]
    pub fn definition(&self) -> TokenStream {
        match self {
            Self::Public => quote! { pub },
            Self::SpecializedPublic(vis) => quote! { pub (@ vis.clone()) },
            Self::Private => TokenStream::new(),
        }
    }
}

impl Indirection {
    #[must_use]
    pub fn ptr_definition(&self) -> TokenStream {
        match self {
            Self::None => TokenStream::new(),
            Self::Constant => quote! { *const },
            Self::Mutable => quote! { *mut },
        }
    }

    #[must_use]
    pub fn wrap_type_ptr(&self, ty: TokenStream) -> TokenStream {
        match self {
            Self::None => ty,
            Self::Constant => quote! { *const (@ ty) },
            Self::Mutable => quote! { *mut (@ ty) },
        }
    }

    #[must_use]
    pub fn wrap_type_ref(&self, ty: TokenStream) -> TokenStream {
        match self {
            Self::None => ty,
            Self::Constant => quote! { &(@ ty) },
            Self::Mutable => quote! { &mut (@ ty) },
        }
    }

    #[must_use]
    pub fn ref_definition(&self) -> TokenStream {
        match self {
            Self::None => TokenStream::new(),
            Self::Constant => quote! { & },
            Self::Mutable => quote! { &mut },
        }
    }
}

impl TypeAliasDefinition {
    /// produces the complete type alias definition, e.g. `type MyType<T> = Vec<T>;`.
    #[must_use]
    pub fn definition(&self) -> TokenStream {
        let generics = self.generics();

        quote! {
            (@ self.visibility.definition()) type (@ self.name().clone()) (@ generics.param_definition()) (@ generics.where_clause_definition()) = (@ self.ty.clone());
        }
    }
}

pub fn impl_item(
    item: &impl ItemDefinition,
    r#trait: Option<TokenStream>,
    body: TokenStream,
) -> TokenStream {
    let generics = item.generics();

    let r#trait = r#trait.map_or_default(|t| quote! { (@ t) for });

    quote! {
        impl (@ generics.param_definition()) (@ generics.where_clause_definition()) (@ r#trait) (@ item.name().clone()) (@ generics.param_usage(None)) {
            (@ body)
        }
    }
}

impl FunctionDefinition {
    /// produces the complete function definition, e.g. `fn my_function<T>(arg: T) -> T where T: ... { ... }`.
    #[must_use]
    pub fn definition(&self) -> TokenStream {
        quote! {
            (@ self.visibility.definition()) fn (@ self.name.clone()) (@ self.generics.param_definition()) (@ self.params_def()) (@ self.return_def()) (@ self.generics.where_clause_definition()) (@ self.body_def())
        }
    }

    fn return_def(&self) -> TokenStream {
        self.ret_ty
            .as_ref()
            .map_or_else(TokenStream::new, |ret_ty| quote! { -> (@ ret_ty.clone()) })
    }

    fn body_def(&self) -> TokenStream {
        self.body
            .as_ref()
            .map_or_else(|| quote! {;}, |body| quote! {{ (@ body.clone()) }})
    }

    fn params_def(&self) -> TokenStream {
        let params = self
            .params
            .iter()
            .map(|param| {
                quote! { (@ param.pattern.clone()): (@ param.ty.clone()), }
            })
            .collect::<TokenStream>();

        quote! {
            ((@ self.self_param_def()) (@ params))
        }
    }

    fn self_param_def(&self) -> TokenStream {
        match self.self_param {
            Some(Indirection::None) => quote! { self },
            Some(Indirection::Constant) => quote! { &self },
            Some(Indirection::Mutable) => quote! { &mut self },
            _ => TokenStream::new(),
        }
    }

    /// produces the complete function signature, e.g. `fn my_function<T>(arg: T) -> T`.
    #[must_use]
    pub fn signature(&self) -> TokenStream {
        quote! {
            fn (@ self.name.clone()) (@ self.generics.param_definition()) (@ self.params_def()) (@ self.return_def())
        }
    }

    /// produces the complete function usage, e.g. `my_function::<T>(arg)`.
    #[must_use]
    pub fn usage(
        &self,
        arg_exprs: &[TokenStream],
        generic_exprs: Option<&[TokenStream]>,
    ) -> TokenStream {
        assert_eq!(
            self.params.len(),
            arg_exprs.len(),
            "Tried to make usage of func with incorrect amount of args"
        );

        let args = arg_exprs
            .iter()
            .cloned()
            .map(|arg| quote! {(@ arg),})
            .collect::<TokenStream>();

        quote! {
            (@ self.name.clone()) (@ self.generics.param_usage(generic_exprs)) ((@ args))
        }
    }
}
