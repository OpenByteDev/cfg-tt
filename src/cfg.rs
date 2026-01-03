use std::collections::{HashMap, HashSet};

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
    // dedup with generate_all_combinations
    pub fn implies(&self, other: &Cfg) -> bool {
        // Fast path.
        if self == other {
            return true;
        }

        // Collect atoms from both sides.
        let atoms = {
            let mut set = HashSet::<Meta>::new();
            self.collect_atoms(&mut set);
            other.collect_atoms(&mut set);
            set.into_iter().collect::<Vec<_>>()
        };

        // Map each atom to an index.
        let mut idx = HashMap::<Meta, usize>::with_capacity(atoms.len());
        for (i, a) in atoms.iter().cloned().enumerate() {
            idx.insert(a, i);
        }

        // Enumerate all boolean assignments. If we find a counterexample where
        // self is true and other is false, implication does not hold.
        let n = atoms.len();
        let mut vals = vec![false; n];

        fn dfs(
            index: usize,
            len: usize,
            assignment: &mut [bool],
            index_map: &HashMap<Meta, usize>,
            left: &Cfg,
            right: &Cfg,
        ) -> bool {
            if index == len {
                let left = left.eval_with(assignment, index_map);
                if !left {
                    return true; // if left is false, we dont care about right
                }
                let right = right.eval_with(assignment, index_map);
                return right; // must be true whenever left is true
            }

            // Try false
            assignment[index] = false;
            if !dfs(index + 1, len, assignment, index_map, left, right) {
                return false;
            }

            // Try true
            assignment[index] = true;
            if !dfs(index + 1, len, assignment, index_map, left, right) {
                return false;
            }

            true
        }

        dfs(0, n, &mut vals, &idx, self, other)
    }

    fn collect_atoms(&self, out: &mut HashSet<Meta>) {
        match self {
            Cfg::Atomic(meta) => {
                out.insert((**meta).clone());
            }
            Cfg::Not(inner) => inner.collect_atoms(out),
            Cfg::Any(vec) | Cfg::All(vec) => {
                for inner in vec {
                    inner.collect_atoms(out);
                }
            }
        }
    }

    fn eval_with(&self, vals: &[bool], index_map: &HashMap<Meta, usize>) -> bool {
        match self {
            Cfg::Atomic(meta) => {
                let index = index_map
                    .get(&(**meta))
                    .expect("atom index missing (collect_atoms/idx bug)");
                vals[*index]
            }
            Cfg::Not(inner) => !inner.eval_with(vals, index_map),
            Cfg::All(vec) => vec.iter().all(|inner| inner.eval_with(vals, index_map)),
            Cfg::Any(vec) => vec.iter().any(|inner| inner.eval_with(vals, index_map)),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(s: &str) -> Meta {
        syn::parse_str::<Meta>(s).unwrap_or_else(|e| panic!("failed to parse Meta `{s}`: {e}"))
    }
    fn atom(name: &str) -> Cfg {
        Cfg::Atomic(Box::new(meta(name)))
    }
    fn any(v: Vec<Cfg>) -> Cfg {
        Cfg::Any(v)
    }
    fn all(v: Vec<Cfg>) -> Cfg {
        Cfg::All(v)
    }
    fn not(x: Cfg) -> Cfg {
        Cfg::Not(Box::new(x))
    }

    #[test]
    fn implies_itself() {
        let a = atom("a");
        assert!(a.implies(&a));
    }

    #[test]
    fn implies_ignores_idenitiy() {
        let a1 = atom("a");
        let a2 = atom("a");
        assert!(a1.implies(&a2));
        assert!(a2.implies(&a1));
    }

    #[test]
    fn distinct_atomics_dont_imply_each_other() {
        let a = atom("a");
        let b = atom("b");
        assert!(!a.implies(&b));
        assert!(!b.implies(&a));
    }

    #[test]
    fn neg_does_not_imply_anything() {
        let a = atom("a");
        let not_a = not(a.clone());
        assert!(!not_a.implies(&a));
        assert!(!a.implies(&not_a));
    }

    #[test]
    fn all_implies_contained() {
        let a = atom("a");
        let b = atom("b");
        let a_and_b = all(vec![a.clone(), b.clone()]);

        assert!(a_and_b.implies(&a));
        assert!(a_and_b.implies(&b));
    }

    #[test]
    fn any_doesnt_imply_contained() {
        let a = atom("a");
        let b = atom("b");
        let a_or_b = any(vec![a.clone(), b.clone()]);

        assert!(!a_or_b.implies(&a));
        assert!(!a_or_b.implies(&b));
    }

    #[test]
    fn child_implies_any() {
        let a = atom("a");
        let b = atom("b");
        let a_or_b = any(vec![a.clone(), b.clone()]);

        assert!(a.implies(&a_or_b));
        assert!(b.implies(&a_or_b));
    }

    #[test]
    fn child_does_not_imply_all() {
        let a = atom("a");
        let b = atom("b");
        let a_and_b = all(vec![a.clone(), b.clone()]);

        assert!(!a.implies(&a_and_b));
        assert!(!b.implies(&a_and_b));
    }

    #[test]
    fn double_negation_is_ignored() {
        let a = atom("a");
        let nna = not(not(a.clone()));

        assert!(a.implies(&nna));
        assert!(nna.implies(&a));
    }

    #[test]
    fn ordering_of_any_does_not_matter() {
        let a = atom("a");
        let b = atom("b");
        let c = atom("c");

        let x1 = any(vec![a.clone(), b.clone(), c.clone()]);
        let x2 = any(vec![c.clone(), a.clone(), b.clone()]);
        assert!(x1.implies(&x2));
        assert!(x2.implies(&x1));
    }

    #[test]
    fn ordering_of_all_does_not_matter() {
        let a = atom("a");
        let b = atom("b");
        let c = atom("c");

        let y1 = all(vec![a.clone(), b.clone(), c.clone()]);
        let y2 = all(vec![c, a, b]);
        assert!(y1.implies(&y2));
        assert!(y2.implies(&y1));
    }

    #[test]
    fn de_morgan_equality() {
        // not(any(a, b)) ⇒ and(not(a), not(b))
        let a = atom("a");
        let b = atom("b");

        let left = not(any(vec![a.clone(), b.clone()]));
        let right = all(vec![not(a), not(b)]);

        assert!(left.implies(&right));
        assert!(right.implies(&left));
    }
}
