use syn::{
    ext::IdentExt,
    parse::ParseStream,
    Ident,
    Token,
};

#[test]
fn test_peek() {
    _ = |input: ParseStream| {
        _ = input.peek(Ident);
        _ = input.peek(Ident::peek_any);
        _ = input.peek(Token![::]);
    };
}
