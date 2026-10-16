use forsith_base::proc_macro::{
    Delimiter, Group, Ident, PunctChar, TokenStream,
    TokenTree::{self},
};

pub fn expand_token_tree(tt: TokenTree) -> TokenTree {
    let TokenTree::Group(group) = tt else {
        return tt;
    };

    let stream = group.stream();
    if group.delimiter != Delimiter::Bracket
        || !stream[0].is_punct(PunctChar::LessThan)
        || !stream[stream.len() - 1].is_punct(PunctChar::GreaterThan)
    {
        let delimiter = group.delimiter;
        let expanded_stream = group
            .take_stream()
            .into_iter()
            .map(expand_token_tree)
            .collect();

        return TokenTree::Group(Group::new(delimiter, expanded_stream));
    }

    let mut stream = group.take_stream().into_iter();
    stream.next();
    let len = stream.len();
    let stream = stream.take(len - 1);

    let mut result = String::new();
    for tt in stream {
        let ident = tt.ident().expect("Expected identifier in paste macro");

        result.push_str(&ident.to_string());
    }

    TokenTree::Ident(Ident::new(&result))
}

pub fn paste(input: TokenStream) -> TokenStream {
    input.into_iter().map(expand_token_tree).collect()
}
