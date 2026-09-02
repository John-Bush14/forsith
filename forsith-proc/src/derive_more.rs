use forsith_shared::casing::{Casing, change_casing};
use proc_macro::{Delimiter, Group, Ident, Punct, Span, TokenStream, TokenTree};
use crate::utils::{ItemType, function_item, impl_item, parse_enum_variants, parse_item};


pub fn derive_is_variant(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter();

    let item = parse_item(&mut input);
    assert_eq!(item.ty(), &ItemType::Enum, "IsVariant can only be derived for enums, found {:?}", item.ty());

    let variants = parse_enum_variants(&mut input);

    let mut functions = TokenStream::new();
    for (variant_ident, _) in &variants {
        let body = TokenStream::from_iter([
            TokenTree::Ident(Ident::new("matches", Span::call_site())),
            TokenTree::Punct(Punct::new('!', proc_macro::Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::from_iter([
                TokenTree::Ident(Ident::new("self", Span::call_site())),
                TokenTree::Punct(Punct::new(',', proc_macro::Spacing::Alone)),
                TokenTree::Ident(Ident::new("Self", Span::call_site())),
                TokenTree::Punct(Punct::new(':', proc_macro::Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', proc_macro::Spacing::Alone)),
                TokenTree::Ident(variant_ident.clone()),
            ]))),
        ]);

        functions.extend(function_item(
            true,
            Ident::new(&format!("is_{}", change_casing(&variant_ident.to_string(), Casing::Snake)), Span::call_site()),
            body,
            Group::new(Delimiter::Parenthesis, TokenStream::from(TokenTree::Ident(Ident::new("self", Span::call_site())))),
            TokenTree::Ident(Ident::new("bool", Span::call_site()))
        ));
    }

    impl_item(&item, functions)
}
