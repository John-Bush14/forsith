use forsith_base::proc_macro::{Delimiter, Ident, Punct, PunctChar, TokenStream, Group, TokenTree::{self}};

pub fn expand_token_tree(tt: TokenTree) -> TokenTree {
    let TokenTree::Group(group) = tt else { return tt };

    let stream = group.stream();
    if group.delimiter != Delimiter::Bracket
        || !matches!(&stream[0], TokenTree::Punct(p) if p.char() == PunctChar::LessThan)
        || !matches!(&stream[stream.len() - 1], TokenTree::Punct(p) if p.char() == PunctChar::GreaterThan)
    {
        let delimiter = group.delimiter;
        let expanded_stream = group.take_stream().into_iter().map(expand_token_tree).collect();

        return TokenTree::Group(Group::new(delimiter, expanded_stream));
    }

    let mut stream = group.take_stream().into_iter();
    stream.next();
    let len = stream.len();
    let stream = stream.take(len - 1);

    let mut result = String::new();
    for tt in stream {
        let TokenTree::Ident(ident) = tt else {
            panic!("Expected identifier in paste macro");
        };

        result.push_str(&ident.to_string());
    }

    TokenTree::Ident(Ident::new(&result))
}

pub fn paste(input: TokenStream) -> TokenStream {
    input.into_iter().map(expand_token_tree).collect()
}
