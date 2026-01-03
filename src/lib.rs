#![doc = include_str!("../README.md")]

mod cfg;
use cfg::*;
mod find;
use find::*;

use std::{collections::HashSet, iter};

use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{
    Item, Stmt,
    parse::{Parse, ParseStream},
};

struct Many<T>(Vec<T>);

impl<T: Parse> Parse for Many<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse::<T>()?);
        }
        Ok(Many(items))
    }
}

fn expand_for_cfg(ts: TokenStream, active_cfg: &Cfg) -> TokenStream {
    let mut it = ts.into_iter().peekable();
    let mut out = TokenStream::new();
    while let Some(tt) = it.next() {
        match &tt {
            TokenTree::Group(g) => {
                let expanded = expand_for_cfg(g.stream(), active_cfg);
                let expanded = TokenTree::Group(Group::new(g.delimiter(), expanded));
                out.extend([expanded]);
            }
            TokenTree::Punct(p) if p.as_char() == '#' => {
                // # ...
                let Some(TokenTree::Group(g)) = it.peek() else {
                    out.extend([tt]);
                    continue;
                };
                if g.delimiter() != Delimiter::Bracket {
                    out.extend([tt]);
                    continue;
                }

                // #[ ... ]
                let mut attr_ts = TokenStream::new();
                attr_ts.extend(iter::once(tt.clone()));
                attr_ts.extend(iter::once(TokenTree::Group(g.clone())));

                let Ok(attr) = parse_any_attr(attr_ts) else {
                    out.extend([tt]);
                    continue;
                };
                let Some(cfg) = Cfg::from_attr(&attr) else {
                    out.extend([tt]);
                    continue;
                };
                if !attr.path().is_ident("cfg") {
                    out.extend([tt]);
                    continue;
                }

                // consume #[cfg(...)]
                let _ = it.next();

                // target of cfg
                let Some(target) = it.next() else { continue };
                if active_cfg.is_active_subset(&cfg) {
                    // active
                    let target = if let TokenTree::Group(g) = target {
                        g.stream()
                    } else {
                        target.into_token_stream()
                    };
                    let expanded = expand_for_cfg(target, active_cfg);
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

fn generate_all_combinations(cfgs: Vec<Cfg>) -> Vec<Cfg> {
    fn core<T: Clone>(
        items: &[T],
        i: usize,
        acc: &mut Vec<(T, bool)>,
        out: &mut impl FnMut(&Vec<(T, bool)>),
    ) {
        if i == items.len() {
            out(acc);
            return;
        }

        // excluded
        acc.push((items[i].clone(), false));
        core(items, i + 1, acc, out);
        acc.pop();

        // included
        acc.push((items[i].clone(), true));
        core(items, i + 1, acc, out);
        acc.pop();
    }

    if cfgs.is_empty() {
        return Vec::new();
    }

    let mut acc = Vec::with_capacity(cfgs.len());
    let mut out = Vec::with_capacity(cfgs.len() * cfgs.len());
    core(&cfgs, 0, &mut acc, &mut |cfgs| {
        let mut list = cfgs
            .iter()
            .cloned()
            .map(
                |(cfg, active)| {
                    if active { cfg } else { Cfg::Not(Box::new(cfg)) }
                },
            )
            .collect::<Vec<_>>();
        out.push(
            if list.len() == 1 {
                list.pop().unwrap()
            } else {
                Cfg::All(list)
            });
    });
    out
}

fn find_base_cfgs(input: impl IntoIterator<Item = Cfg>) -> Vec<Cfg> {
    let mut cfgs = HashSet::new();

    // Remove duplicates and negations
    for cfg in input.into_iter() {
        match cfg {
            Cfg::Not(inner) => cfgs.insert(*inner),
            Cfg::All(list) | Cfg::Any(list) if list.is_empty() => false,
            Cfg::All(list) | Cfg::Any(list) if list.len() == 1 => cfgs.insert(list[0].clone()),
            _ => cfgs.insert(cfg),
        };
    }

    // Remove all() if all inner cfgs exist
    let cfgs: Vec<Cfg> = cfgs
        .iter()
        .filter(|cfg| match cfg {
            Cfg::All(xs) => !xs.iter().all(|child| match child {
                Cfg::Not(inner) => cfgs.contains(inner),
                _ => cfgs.contains(child),
            }),
            _ => true,
        })
        .cloned()
        .collect();

    cfgs
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

    // Collect all occurances or #[cfg()] in the input
    let cfgs = find_cfg_attrs(content.clone());
    if cfgs.is_empty() {
        // Nothing to do
        return content.into();
    }

    let cfgs = find_base_cfgs(cfgs);

    // Now construct every possible combination of applicable configurations
    let configurations = generate_all_combinations(cfgs);

    let mut out = TokenStream::new();
    for cfg in &configurations {
        let expanded = expand_for_cfg(content.clone(), cfg);
        let items = match syn::parse2::<Many<Item>>(expanded.clone()) {
            Ok(items) => items.0.iter().map(|item| item.to_token_stream()).collect(),
            Err(_) => match syn::parse2::<Many<Stmt>>(expanded.clone()) {
                Ok(stmts) => stmts.0.iter().map(|item| item.to_token_stream()).collect(),
                Err(_) => vec![expanded],
            },
        };

        for item in items {
            out.extend([cfg.to_token_stream(), item]);
        }
    }

    // panic!("{}", out.to_string());

    out.into()
}
