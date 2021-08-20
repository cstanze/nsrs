extern crate proc_macro;
use proc_macro::TokenStream as TS1;
use quote::{ToTokens, format_ident, quote, TokenStreamExt};
use proc_macro2::{TokenTree, Spacing, Span, TokenStream, token_stream::IntoIter as TokenIter, Ident};
use syn::{Attribute, Data, DeriveInput, Fields, parse_macro_input, spanned::Spanned};

fn is_punct(tt: &TokenTree, expect: char) -> bool {
    match tt {
        TokenTree::Punct(punct)
            if punct.as_char() == expect && punct.spacing() == Spacing::Alone =>
        {
            true
        }
        _ => false,
    }
}

/// If supplied `tt` is a punct matching a char, returns `None`, else returns `tt`
fn expect_punct(tt: Option<TokenTree>, expect: char) -> Option<TokenTree> {
    tt.filter(|tt| !is_punct(&tt, expect))
}

enum NestedValue {
    /// `name = ...`
    Assign(TokenStream),
    /// `name(...)`
    Group(TokenStream),
    /// `name ident = ...`
    KeywordAssign(Ident, TokenStream),
}

enum Nested {
    /// Unnamed nested attribute, such as a string,
    /// callback closure, or a lone ident/path
    ///
    /// Note: a lone ident will be Named with no value instead
    Unnamed(TokenStream),
    /// Named: name ...
    Named(Ident, NestedValue),
    /// Unexpected token,
    Unexpected(TokenStream),
}

struct AttributeParser {
    inner: TokenIter,
}

struct Empty;

impl From<Empty> for TokenStream {
    fn from(_: Empty) -> TokenStream {
        TokenStream::new()
    }
}

impl AttributeParser {
    pub fn new(stream: TokenStream) -> Self {
        AttributeParser {
            inner: stream.into_iter(),
        }
    }

    fn next_tt(&mut self) -> Option<TokenTree> {
        expect_punct(self.inner.next(), ',')
    }

    fn collect_tail<T>(&mut self, first: T) -> TokenStream
    where
        T: Into<TokenStream>,
    {
        let mut out = first.into();

        while let Some(tt) = self.next_tt() {
            out.extend(Some(tt));
        }

        out
    }

    fn parse_unnamed(&mut self, first: Ident, next: TokenTree) -> Nested {
        let mut out = TokenStream::from(TokenTree::Ident(first));

        out.extend(self.collect_tail(next));

        Nested::Unnamed(out.into_iter().collect())
    }

    fn parse_assign(&mut self, name: Ident) -> Nested {
        let value = self.collect_tail(Empty);

        Nested::Named(name, NestedValue::Assign(value))
    }

    fn parse_group(&mut self, name: Ident, group: TokenStream) -> Nested {
        Nested::Named(name, NestedValue::Group(group))
    }

    fn parse_keyword(&mut self, keyword: Ident, name: Ident) -> Nested {
        let error = expect_punct(self.next_tt(), '=');

        match error {
            Some(error) => {
                let error = self.collect_tail(error);

                Nested::Unexpected(error)
            }
            None => {
                let value = self.collect_tail(Empty);

                Nested::Named(keyword, NestedValue::KeywordAssign(name, value))
            }
        }
    }
}

impl Iterator for AttributeParser {
    type Item = Nested;

    fn next(&mut self) -> Option<Nested> {
        let first = self.inner.next()?;

        let name = match first {
            TokenTree::Ident(ident) => ident,
            tt => {
                let stream = self.collect_tail(tt);

                return Some(Nested::Unnamed(stream.into_iter().collect()));
            }
        };

        match self.next_tt() {
            Some(tt) if is_punct(&tt, '=') => Some(self.parse_assign(name)),
            Some(TokenTree::Group(group)) => Some(self.parse_group(name, group.stream())),
            Some(TokenTree::Ident(next)) => Some(self.parse_keyword(name, next)),
            Some(next) => Some(self.parse_unnamed(name, next)),
            None => Some(Nested::Unnamed(quote!(#name))),
        }
    }
}

#[derive(Clone, Debug)]
enum Callback {
    Label(TokenStream),
    Inline(Box<InlineCallback>),
    None
}

#[derive(Clone, Debug)]
struct InlineCallback {
    pub arg: Ident,
    pub body: TokenStream,
    pub span: Span,
}

impl From<InlineCallback> for Callback {
    fn from(inline: InlineCallback) -> Callback {
        Callback::Inline(Box::new(inline))
    }
}

impl Callback {
    pub fn token_stream(&self) -> TokenStream {
        match self {
            Callback::Label(t) => t.to_owned(),
            Callback::Inline(t) => t.body.to_owned(),
            Callback::None => TokenStream::new()
        }
    }
}

impl ToTokens for Callback {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for token in self.token_stream().into_iter() {
            tokens.append(token);
        }
    }
}

struct Parser;

impl Parser {
    fn parse_attr(attr: &mut Attribute) -> Option<AttributeParser> {
        let mut tokens = std::mem::replace(&mut attr.tokens, TokenStream::new()).into_iter();

        match tokens.next() {
            Some(TokenTree::Group(group)) => Some(AttributeParser::new(group.stream())),
            _ => None,
        }
    }

    fn parse_callback(tokens: TokenStream) -> Option<Callback> {
        let span = tokens.span();
        let mut tokens = tokens.into_iter();

        if let Some(tt) = expect_punct(tokens.next(), '|') {
            let mut label = TokenStream::from(tt);

            label.extend(tokens);

            return Some(Callback::Label(label));
        }

        let first = tokens.next();
        let error = expect_punct(tokens.next(), '|');

        let arg = match (error, first) {
            (None, Some(TokenTree::Ident(arg))) => arg,
            _ => {
                panic!("Inline callbacks must use closure syntax with exactly one parameter {:?}", span);
            }
        };

        let body = match tokens.next() {
            Some(TokenTree::Group(group)) => group.stream(),
            Some(first) => {
                let mut body = TokenStream::from(first);

                body.extend(tokens);
                body
            }
            None => {
                panic!("Callback missing a body {:?}", span);
            }
        };

        let inline = InlineCallback { arg, body, span };

        Some(inline.into())
    }
}

#[proc_macro_derive(GenericASTNode, attributes(children, node_type))]
pub fn derive_generic_ast_node(input: TS1) -> TS1 {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = format_ident!("{}", input.ident);
    let mut ext = format_ident!("_");
    let mut ecb = Callback::None;
    
    match input.data {
        Data::Struct(s) => {
            match s.fields {
                Fields::Named(mut f) => {
                    for field in &mut f.named {
                        for attr in &mut field.attrs {
                            if attr.path.is_ident("children") {
                                ext = attr.path.segments[0].ident.clone();
                            } else if attr.path.is_ident("node_type") {
                                let nested = match Parser::parse_attr(attr) {
                                    None => {
                                        panic!("GenericASTNode derive must include a #[node_type] attribute on the struct")
                                    },
                                    Some(tokens) => tokens
                                };

                                for (pos, next) in nested.enumerate() {
                                    match next {
                                        Nested::Unnamed(tokens) => match pos {
                                            0 => ecb = match Parser::parse_callback(tokens) {
                                                Some(cb) => cb,
                                                None => panic!("Invalid callback")
                                            },
                                            _ => panic!("Expected named argument")
                                        },
                                        _ => panic!("Unexpected token")
                                    }
                                }
                            }
                        }
                    }
                },
                _ => {}
            }
        },
        _ => {}
    };

    if ext == format_ident!("_") {
        panic!("GenericASTNode derive must include a #[children] attribute on one field")
    }

    match ecb {
        Callback::None => {
            panic!("GenericASTNode derive must include a #[node_type] attribute with a callback for the node type")
        }
        _ => {}
    };

    let expanded = quote! {
        impl ASTNode for #ident {
            fn debug_fmt(&self) -> String {
                format!("{:#?}", self)
            }

            fn node_type(&self) -> &str {
                (#ecb)()
            }

            fn get_leaves(&self) -> &Vec<Box<dyn ASTNode>> {
                &self.#ext
            }

            fn push_leaf(&mut self, data: Box<dyn ASTNode>) {
                self.#ext.push(data)
            }

            fn get_leaf_mut(&mut self, index: usize) -> Option<&mut Box<dyn ASTNode>> {
                self.#ext.get_mut(index)
            }
        }
    };

    TS1::from(expanded)
}
