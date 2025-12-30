#![doc = include_str!("../README.md")]

use std::{collections::HashSet, iter};

use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::ToTokens;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::{Attribute, Item};

struct AnyAttribute(Attribute);

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

struct Items(Vec<Item>);

impl Parse for Items {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse::<Item>()?);
        }
        Ok(Items(items))
    }
}

fn parse_any_attr(ts: TokenStream) -> syn::Result<Attribute> {
    syn::parse2::<AnyAttribute>(ts).map(|a| a.0)
}

fn find_cfg_attrs(ts: TokenStream) -> HashSet<Attribute> {
    fn core(ts: TokenStream, out: &mut HashSet<Attribute>) {
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
                    if !attr.path().is_ident("cfg") {
                        continue;
                    }

                    // #[cfg(...)]
                    out.insert(attr);
                    let _ = it.next();
                }
                _ => {}
            }
        }
    }

    let mut out = HashSet::new();
    core(ts, &mut out);
    out
}

fn expand_for_cfg(ts: TokenStream, cfg: &Attribute) -> TokenStream {
    let mut it = ts.into_iter().peekable();
    let mut out = TokenStream::new();
    while let Some(tt) = it.next() {
        match &tt {
            TokenTree::Group(g) => {
                let expanded = expand_for_cfg(g.stream(), cfg);
                let expanded = TokenTree::Group(Group::new(g.delimiter(), expanded));
                out.extend([expanded]);
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
                if !attr.path().is_ident("cfg") {
                    continue;
                }

                // #[cfg(...)]
                let _ = it.next();

                // target of cfg
                let Some(target) = it.next() else { continue };
                if cfg == &attr {
                    // active
                    let target = if let TokenTree::Group(g) = target {
                        g.stream()
                    } else {
                        target.into_token_stream()
                    };
                    let expanded = expand_for_cfg(target, cfg);
                    out.extend([expanded]);
                } else {
                    // dont emit anything
                }
            }
            _ => {
                out.extend([tt]);
            }
        }
    }

    out
}

/// Apply `#[cfg(...)]` at **token-tree granularity**, anywhere.
///
/// This macro processes raw token trees *before parsing* and allows
/// `#[cfg]` to appear in places Rust normally forbids.
///
/// Each `#[cfg(...)]` attribute applies to **exactly the next `TokenTree`**:
/// - an identifier (e.g. `foo`)
/// - a literal (e.g. `42`)
/// - a punctuation token (e.g. `+`)
/// - a group (`{}`, `()`, `[]`)
///
/// To conditionally include more than one token tree, wrap them in a group.
///
/// After cfg filtering, the remaining tokens are emitted unchanged and must
/// form valid Rust code.
#[proc_macro]
pub fn cfg_tt(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let content: TokenStream = input.into();
    let cfgs = find_cfg_attrs(content.clone());

    let mut out = TokenStream::new();
    for cfg in &cfgs {
        let expanded = expand_for_cfg(content.clone(), cfg);
        let items = match syn::parse2::<Items>(expanded.clone()) {
            Ok(items) => items.0.iter().map(|item| item.to_token_stream()).collect(),
            Err(_) => vec![expanded],
        };
        for item in items {
            out.extend([cfg.into_token_stream(), item]);
        }
    }

    out.into()
}
