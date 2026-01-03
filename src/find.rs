use std::iter;

use crate::cfg::Cfg;
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use syn::{
    Attribute,
    parse::{Parse, ParseStream},
};

pub struct AnyAttribute(Attribute);

impl Parse for AnyAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let attr = Attribute {
            pound_token: input.parse()?,
            style: input
                .parse::<syn::token::Not>()
                .ok()
                .map_or_else(|| syn::AttrStyle::Outer, syn::AttrStyle::Inner),
            bracket_token: syn::bracketed!(content in input),
            meta: content.parse()?,
        };
        Ok(Self(attr))
    }
}

pub fn parse_any_attr(ts: TokenStream) -> syn::Result<Attribute> {
    syn::parse2::<AnyAttribute>(ts).map(|a| a.0)
}

pub fn find_cfg_attrs(ts: TokenStream) -> Vec<Cfg> {
    fn core(ts: TokenStream, out: &mut Vec<Cfg>) {
        let mut it = ts.into_iter().peekable();

        while let Some(tt) = it.next() {
            match &tt {
                TokenTree::Group(g) => {
                    core(g.stream(), out);
                }
                TokenTree::Punct(p) if p.as_char() == '#' => {
                    // # ...
                    let Some(TokenTree::Group(g)) = it.peek() else {
                        continue;
                    };
                    if g.delimiter() != Delimiter::Bracket {
                        continue;
                    }

                    // #[ ... ]
                    let mut attr_ts = TokenStream::new();
                    attr_ts.extend(iter::once(tt.clone()));
                    attr_ts.extend(iter::once(TokenTree::Group(g.clone())));

                    let Ok(attr) = parse_any_attr(attr_ts) else {
                        continue;
                    };
                    if let Some(cfg) = Cfg::from_attr(&attr) {
                        // #[cfg(...)]
                        out.push(cfg);
                        let _ = it.next();
                    }
                }
                _ => {}
            }
        }
    }

    let mut out = Vec::new();
    core(ts, &mut out);
    out
}
