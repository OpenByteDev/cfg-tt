use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Meta, MetaList, Token, punctuated::Punctuated};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Cfg {
    Any(Vec<Cfg>),
    All(Vec<Cfg>),
    Not(Box<Cfg>),
    Atomic(Box<Meta>),
}

impl Cfg {
    pub fn is_active_subset(&self, other: &Cfg) -> bool {
        use Cfg::*;

        // identical structure fast path
        if self == other {
            return true;
        }

        match (self, other) {
            (Atomic(a), Atomic(b)) => a == b,

            (All(xs), y) => xs.iter().all(|x| x.is_active_subset(y)),
            (x, All(ys)) => ys.iter().all(|y| x.is_active_subset(y)),

            (Any(xs), y) => xs.iter().all(|x| x.is_active_subset(y)),
            (x, Any(ys)) => ys.iter().any(|y| x.is_active_subset(y)),

            (Not(a), Not(b)) => b.is_active_subset(a),

            _ => false,
        }
    }
}

impl Cfg {
    pub fn from_attr(attr: &Attribute) -> Option<Cfg> {
        if !attr.path().is_ident("cfg") {
            return None;
        }

        let Meta::List(list) = &attr.meta else {
            return None;
        };

        // cfg(...) must contain exactly one meta item; cfg(a, b) => ignore
        let items: Punctuated<Meta, Token![,]> = list
            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .ok()?;

        if items.len() != 1 {
            return None;
        }

        Self::from_cfg_meta(items.into_iter().next().unwrap())
    }

    fn from_cfg_meta(meta: Meta) -> Option<Cfg> {
        match meta {
            Meta::List(list) if list.path.is_ident("any") => {
                let args = Self::from_cfg_args(list)?;
                Some(Cfg::Any(args))
            }
            Meta::List(list) if list.path.is_ident("all") => {
                let args = Self::from_cfg_args(list)?;
                Some(Cfg::All(args))
            }
            Meta::List(list) if list.path.is_ident("not") => {
                let args = Self::from_cfg_args(list)?;
                if args.len() != 1 {
                    return None; // not() or not(a,b) => ignore
                }
                Some(Cfg::Not(Box::new(args.into_iter().next().unwrap())))
            }
            other => Some(Cfg::Atomic(Box::new(other))),
        }
    }

    fn from_cfg_args(list: MetaList) -> Option<Vec<Cfg>> {
        let nested: Punctuated<Meta, Token![,]> = list
            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .ok()?;

        nested.into_iter().map(Self::from_cfg_meta).collect()
    }
}

impl Cfg {
    pub fn to_token_stream(&self) -> TokenStream {
        let pred = self.to_cfg_meta();
        quote!(#[cfg(#pred)])
    }

    fn to_cfg_meta(&self) -> TokenStream {
        match self {
            Cfg::Any(xs) => {
                let inner = xs.iter().map(|c| c.to_cfg_meta());
                quote!(any(#(#inner),*))
            }
            Cfg::All(xs) => {
                let inner = xs.iter().map(|c| c.to_cfg_meta());
                quote!(all(#(#inner),*))
            }
            Cfg::Not(x) => {
                let inner = x.to_cfg_meta();
                quote!(not(#inner))
            }
            Cfg::Atomic(meta) => quote!(#meta),
        }
    }
}
