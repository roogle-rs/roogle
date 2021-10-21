use std::{
    cmp::{max, min},
    collections::HashMap,
};

use levenshtein::levenshtein;
use rustdoc_types as types;
use tracing::{instrument, trace};

use crate::query::*;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Similarity {
    /// Represents how digitally similar two objects are.
    Discrete(DiscreteSimilarity),

    /// Represents how analogly similar two objects are.
    Continuous(f32),
}

impl Similarity {
    pub fn score(&self) -> f32 {
        match self {
            Discrete(Equivalent) => 0.0,
            Discrete(Subequal) => 0.25,
            Discrete(Different) => 1.0,
            Continuous(s) => *s,
        }
    }
}

use Similarity::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Similarities(pub Vec<Similarity>);

impl Similarities {
    /// Calculate objective similarity for sorting.
    pub fn score(&self) -> f32 {
        let sum: f32 = self.0.iter().map(|sim| sim.score()).sum();
        sum / self.0.len() as f32
    }
}

impl PartialOrd for Similarities {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (self.score()).partial_cmp(&other.score())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiscreteSimilarity {
    /// Indicates that two types are the same.
    ///
    /// For example:
    /// - `i32` and `i32`
    /// - `Result<i32, ()>` and `Result<i32, ()>`
    Equivalent,

    /// Indicates that two types are partially equal.
    ///
    /// For example:
    /// - an unbound generic type `T` and `i32`
    /// - an unbound generic type `T` and `Option<U>`
    Subequal,

    /// Indicates that two types are not similar at all.
    ///
    /// For example:
    /// - `i32` and `Option<bool>`
    Different,
}

use DiscreteSimilarity::*;

pub trait Compare<Rhs> {
    fn compare(
        &self,
        rhs: &Rhs,
        krate: &types::Crate,
        generics: &mut types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity>;
}

impl Compare<types::Item> for Query {
    #[instrument(skip(krate))]
    fn compare(
        &self,
        item: &types::Item,
        krate: &types::Crate,
        generics: &mut types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        let mut sims = vec![];

        match (&self.name, &item.name) {
            (Some(q), Some(i)) => sims.append(&mut q.compare(i, krate, generics, substs)),
            (Some(_), None) => sims.push(Discrete(Different)),
            _ => {}
        }
        trace!(?sims);

        if let Some(ref kind) = self.kind {
            sims.append(&mut kind.compare(&item.inner, krate, generics, substs))
        }
        trace!(?sims);

        sims
    }
}

impl Compare<String> for Symbol {
    #[instrument]
    fn compare(
        &self,
        symbol: &String,
        _: &types::Crate,
        _: &mut types::Generics,
        _: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        use std::cmp::max;
        vec![Continuous(
            levenshtein(self, symbol) as f32 / max(self.len(), symbol.len()) as f32,
        )]
    }
}

impl Compare<types::ItemEnum> for QueryKind {
    #[instrument(skip(krate))]
    fn compare(
        &self,
        kind: &types::ItemEnum,
        krate: &types::Crate,
        generics: &mut types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        use types::ItemEnum::*;
        use QueryKind::*;

        match (self, kind) {
            (FunctionQuery(q), Function(i)) => q.compare(i, krate, generics, substs),
            (FunctionQuery(q), Method(i)) => q.compare(i, krate, generics, substs),
            (FunctionQuery(_), _) => vec![Discrete(Different)],
        }
    }
}

impl Compare<types::Function> for Function {
    #[instrument(skip(krate))]
    fn compare(
        &self,
        function: &types::Function,
        krate: &types::Crate,
        generics: &mut types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        generics
            .params
            .append(&mut function.generics.params.clone());
        generics
            .where_predicates
            .append(&mut function.generics.where_predicates.clone());
        self.decl.compare(&function.decl, krate, generics, substs)
    }
}

impl Compare<types::Method> for Function {
    #[instrument(skip(krate))]
    fn compare(
        &self,
        method: &types::Method,
        krate: &types::Crate,
        generics: &mut types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        generics.params.append(&mut method.generics.params.clone());
        generics
            .where_predicates
            .append(&mut method.generics.where_predicates.clone());
        self.decl.compare(&method.decl, krate, generics, substs)
    }
}

impl Compare<types::FnDecl> for FnDecl {
    #[instrument(skip(krate))]
    fn compare(
        &self,
        decl: &types::FnDecl,
        krate: &types::Crate,
        generics: &mut types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        let mut sims = vec![];

        if let Some(ref inputs) = self.inputs {
            inputs.iter().enumerate().for_each(|(idx, q)| {
                if let Some(i) = decl.inputs.get(idx) {
                    sims.append(&mut q.compare(i, krate, generics, substs))
                }
            });

            if inputs.len() != decl.inputs.len() {
                // FIXME: Replace this line below with `usize::abs_diff` once it got stablized.
                let abs_diff =
                    max(inputs.len(), decl.inputs.len()) - min(inputs.len(), decl.inputs.len());
                sims.append(&mut vec![Discrete(Different); abs_diff])
            } else if inputs.is_empty() && decl.inputs.is_empty() {
                sims.push(Discrete(Equivalent));
            }
        }
        trace!(?sims);

        if let Some(ref output) = self.output {
            sims.append(&mut output.compare(&decl.output, krate, generics, substs));
        }
        trace!(?sims);

        sims
    }
}

impl Compare<(String, types::Type)> for Argument {
    #[instrument(skip(krate))]
    fn compare(
        &self,
        arg: &(String, types::Type),
        krate: &types::Crate,
        generics: &mut types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        let mut sims = vec![];

        if let Some(ref name) = self.name {
            sims.append(&mut name.compare(&arg.0, krate, generics, substs));
        }
        trace!(?sims);

        if let Some(ref type_) = self.ty {
            sims.append(&mut type_.compare(&arg.1, krate, generics, substs));
        }
        trace!(?sims);

        sims
    }
}

impl Compare<Option<types::Type>> for FnRetTy {
    #[instrument(skip(krate))]
    fn compare(
        &self,
        ret_ty: &Option<types::Type>,
        krate: &types::Crate,
        generics: &mut types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        match (self, ret_ty) {
            (FnRetTy::Return(q), Some(i)) => q.compare(i, krate, generics, substs),
            (FnRetTy::DefaultReturn, None) => vec![Discrete(Equivalent)],
            _ => vec![Discrete(Different)],
        }
    }
}

fn compare_type(
    lhs: &Type,
    rhs: &types::Type,
    krate: &types::Crate,
    generics: &mut types::Generics,
    substs: &mut HashMap<String, Type>,
    allow_recursion: bool,
) -> Vec<Similarity> {
    use {crate::query::Type::*, types::Type};

    match (lhs, rhs) {
        (q, Type::Generic(i)) if i == "Self" => {
            let mut i = None;
            for where_predicate in &generics.where_predicates {
                if let types::WherePredicate::EqPredicate {
                    lhs: Type::Generic(lhs),
                    rhs,
                } = where_predicate
                {
                    if lhs == "Self" {
                        i = Some(rhs).cloned();
                        break;
                    }
                }
            }
            let i = &i.unwrap(); // SAFETY: `Self` only appears in definitions of associated items.
            q.compare(i, krate, generics, substs)
        }
        (q, Type::Generic(i)) => match substs.get(i) {
            Some(i) => {
                if q == i {
                    vec![Discrete(Equivalent)]
                } else {
                    vec![Discrete(Different)]
                }
            }
            None => {
                substs.insert(i.clone(), q.clone());
                vec![Discrete(Subequal)]
            }
        },
        (q, Type::ResolvedPath { id, .. })
            if krate
                .index
                .get(id)
                .map(|i| matches!(i.inner, types::ItemEnum::Typedef(_)))
                .unwrap_or(false)
                && allow_recursion =>
        {
            let sims_typedef = compare_type(lhs, rhs, krate, generics, substs, false);
            if let Some(types::Item {
                inner: types::ItemEnum::Typedef(types::Typedef { type_: ref i, .. }),
                ..
            }) = krate.index.get(id)
            {
                // TODO: Acknowledge `generics` of `types::Typedef` to get more accurate search results.
                let sims_adt = q.compare(i, krate, generics, substs);
                let sum =
                    |sims: &Vec<Similarity>| -> f32 { sims.iter().map(Similarity::score).sum() };
                if sum(&sims_adt) < sum(&sims_typedef) {
                    return sims_adt;
                }
            }
            sims_typedef
        }
        (Tuple(q), Type::Tuple(i)) => {
            let mut sims = q
                .iter()
                .zip(i.iter())
                .filter_map(|(q, i)| q.as_ref().map(|q| q.compare(i, krate, generics, substs)))
                .flatten()
                .collect::<Vec<_>>();

            // They are both tuples.
            sims.push(Discrete(Equivalent));

            // FIXME: Replace this line below with `usize::abs_diff` once it got stablized.
            let abs_diff = max(q.len(), i.len()) - min(q.len(), i.len());
            sims.append(&mut vec![Discrete(Different); abs_diff]);

            sims
        }
        (Slice(q), Type::Slice(i)) => {
            // They are both slices.
            let mut sims = vec![Discrete(Equivalent)];

            if let Some(q) = q {
                sims.append(&mut q.compare(i, krate, generics, substs));
            }

            sims
        }
        (
            RawPointer {
                mutable: q_mut,
                type_: q,
            },
            Type::RawPointer {
                mutable: i_mut,
                type_: i,
            },
        )
        | (
            BorrowedRef {
                mutable: q_mut,
                type_: q,
            },
            Type::BorrowedRef {
                mutable: i_mut,
                type_: i,
                ..
            },
        ) => {
            if q_mut == i_mut {
                q.compare(i, krate, generics, substs)
            } else {
                let mut sims = q.compare(i, krate, generics, substs);
                sims.push(Discrete(Subequal));
                sims
            }
        }
        (q, Type::RawPointer { type_: i, .. } | Type::BorrowedRef { type_: i, .. }) => {
            let mut sims = q.compare(i, krate, generics, substs);
            sims.push(Discrete(Subequal));
            sims
        }
        (RawPointer { type_: q, .. } | BorrowedRef { type_: q, .. }, i) => {
            let mut sims = q.compare(i, krate, generics, substs);
            sims.push(Discrete(Subequal));
            sims
        }
        (
            UnresolvedPath {
                name: q,
                args: q_args,
            },
            Type::ResolvedPath {
                name: i,
                args: i_args,
                ..
            },
        ) => {
            let mut sims = q.compare(i, krate, generics, substs);

            match (q_args, i_args) {
                (Some(q), Some(i)) => match (&**q, &**i) {
                    (
                        GenericArgs::AngleBracketed { args: ref q },
                        types::GenericArgs::AngleBracketed { args: ref i, .. },
                    ) => {
                        let q = q.iter().map(|q| {
                            q.as_ref().map(|q| match q {
                                GenericArg::Type(q) => q,
                            })
                        });
                        let i = i.iter().map(|i| match i {
                            types::GenericArg::Type(t) => Some(t),
                            _ => None,
                        });
                        q.zip(i).for_each(|(q, i)| match (q, i) {
                            (Some(q), Some(i)) => {
                                sims.append(&mut q.compare(i, krate, generics, substs))
                            }
                            (Some(_), None) => sims.push(Discrete(Different)),
                            (None, _) => {}
                        });
                    }
                    // TODO: Support `GenericArgs::Parenthesized`.
                    (_, _) => {}
                },
                (Some(q), None) => {
                    let GenericArgs::AngleBracketed { args: ref q } = **q;
                    sims.append(&mut vec![Discrete(Different); q.len()])
                }
                (None, _) => {}
            }

            sims
        }
        (Primitive(q), Type::Primitive(i)) => q.compare(i, krate, generics, substs),
        _ => vec![Discrete(Different)],
    }
}

impl Compare<types::Type> for Type {
    #[instrument(skip(krate))]
    fn compare(
        &self,
        type_: &types::Type,
        krate: &types::Crate,
        generics: &mut types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        compare_type(self, type_, krate, generics, substs, true)
    }
}

impl Compare<String> for PrimitiveType {
    #[instrument]
    fn compare(
        &self,
        prim_ty: &String,
        _: &types::Crate,
        _: &mut types::Generics,
        _: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        if self.as_str() == prim_ty {
            vec![Discrete(Equivalent)]
        } else {
            vec![Discrete(Different)]
        }
    }
}
