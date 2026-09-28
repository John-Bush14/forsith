use crate::{
    proc_macro::{
        Punct, PunctChar, TokenStream, TokenTree, items::{FunctionDefinition, GenericDefinition, GenericsDefinition, Indirection, ItemDefinition, TypeAliasDefinition, Visibility},
    }, quote,
};

impl GenericsDefinition {
    /// produces the complete generic definition for an item, e.g. `<T, U>` or `<T: Clone>`.
    #[must_use]
    pub fn definition(&self) -> TokenStream {
        if self.0.is_empty() {
            return TokenStream::new();
        }

        quote! {
            < (@ self.generic_definitions()) >
        }
    }

    /// produces the inner generic definitions for an item without < and >, e.g. `T, U` or `T: Clone`.
    /// Always has a leading comma if there are any generic definitions, so that it can be used in
    /// combination with other generic definitions.
    #[must_use]
    pub fn generic_definitions(&self) -> TokenStream {
        self.0
            .iter()
            .map(|generic| quote! {(@ generic.definition()),})
            .collect()
    }

    /// produces the usage of the generics for an item, e.g. `<T, U>` or `<T>`.
    #[must_use]
    pub fn usage(&self) -> TokenStream {
        if self.0.is_empty() {
            return TokenStream::new();
        }

        quote! {
            < (@ self.generic_usage()) >
        }
    }

    /// produces the inner generic usages for an item without < and >, e.g. `T, U` or `T`.
    /// Always has a leading comma if there are any generic usages, so that it can be used in
    /// combination with other generic usages.
    #[must_use]
    pub fn generic_usage(&self) -> TokenStream {
        self.0
            .iter()
            .map(|generic| quote! {(@ generic.usage()),})
            .collect()
    }
}

impl GenericDefinition {
    /// produces the complete generic definition for a single generic, e.g. `T` or `T: Clone`.
    #[must_use]
    pub fn definition(&self) -> TokenStream {
        match self {
            Self::Lifetime(ident) => TokenStream::from_iter([
                TokenTree::Punct(Punct::new(PunctChar::Qoute, true)),
                TokenTree::Ident(ident.clone()),
            ]),
            Self::Type(ident, bounds) => quote! {
                (@ ident.clone()) : (@ bounds.clone())
            },
        }
    }

    /// produces the usage of a single generic, e.g. `T` or `T` (without bounds).
    #[must_use]
    pub fn usage(&self) -> TokenStream {
        match self {
            Self::Lifetime(ident) => TokenStream::from_iter([
                TokenTree::Punct(Punct::new(PunctChar::Qoute, true)),
                TokenTree::Ident(ident.clone()),
            ]),
            Self::Type(ident, _) => quote! {
                (@ ident.clone())
            },
        }
    }
}

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

impl TypeAliasDefinition {
    /// produces the complete type alias definition, e.g. `type MyType<T> = Vec<T>;`.
    #[must_use]
    pub fn definition(&self) -> TokenStream {
        let generics = self.generics();

        quote! {
            (@ self.visibility.definition()) type (@ self.name().clone()) (@ generics.definition()) = (@ self.ty.clone());
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
        impl (@ generics.definition()) (@ r#trait) (@ item.name().clone()) (@ generics.usage()) {
            (@ body)
        }
    }
}

impl FunctionDefinition {
    /// produces the complete function definition, e.g. `fn my_function<T>(arg: T) -> T { ... }`.
    #[must_use]
    pub fn definition(&self) -> TokenStream {
        quote! {
            (@ self.visibility.definition()) fn (@ self.name.clone()) (@ self.generics.definition()) (@ self.params_def()) (@ self.return_def()) (@ self.body_def())
        }
    }

    fn return_def(&self) -> TokenStream {
        self.ret_ty.as_ref().map_or_else(|| TokenStream::new(), |ret_ty| quote! { -> (@ ret_ty.clone()) })
    }

    fn body_def(&self) -> TokenStream {
        self.body.as_ref().map_or_else(|| TokenStream::new(), |body| quote! {{ (@ body.clone()) }})
    }

    fn params_def(&self) -> TokenStream {
        let params = self.params.iter().map(|param| {
            quote! { (@ param.0.clone()): (@ param.1.clone()), }
        }).collect::<TokenStream>();

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

    fn params_usage(&self) -> TokenStream {
        let params = self.params.iter().map(|param| {
            quote! { (@ param.0.clone()), }
        }).collect::<TokenStream>();

        quote! {
            ((@ self.self_param_def()) (@ params))
        }
    }

    /// produces the complete function signature, e.g. `fn my_function<T>(arg: T) -> T`.
    #[must_use]
    pub fn signature(&self) -> TokenStream {
        quote! {
            fn (@ self.name.clone()) (@ self.generics.definition()) (@ self.params_def()) (@ self.return_def())
        }
    }

    /// produces the complete function usage, e.g. `my_function::<T>(arg)`.
    #[must_use]
    pub fn usage(&self) -> TokenStream {
        let generics = self.generics();

        quote! {
            (@ self.name.clone()) (@ generics.usage()) (@ self.params_usage())
        }
    }
}
