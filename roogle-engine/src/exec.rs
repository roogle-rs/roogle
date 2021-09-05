use std::collections::HashMap;

use rustdoc_types as types;
use rustdoc_types::{Generics, Item, ItemEnum, WherePredicate};

use crate::approx::{Approximate, Similarity};
use crate::types::{Crates, Query, Type};

pub struct QueryExecutor {
    krates: Crates,
}

impl QueryExecutor {
    pub fn new(krates: Crates) -> Self {
        Self { krates }
    }

    pub fn exec(&self, query: Query) -> Vec<&Item> {
        let mut items_with_sims = Vec::new();
        for krate in self.krates.krates.values() {
            for function in krate.functions.values() {
                let sims = query.approx(function, &Generics::default(), &mut HashMap::new());
                if sims.iter().any(|sim| sim != &Similarity::Different) {
                    items_with_sims.push((function, sims))
                }
            }
        }

        if let Some(name) = query
            .args()
            .as_ref()
            .and_then(|args| args.first())
            .and_then(|arg| arg.ty.as_ref())
            .and_then(|ty| {
                let ty = ty.inner_type();
                match ty {
                    Type::UnresolvedPath { name, .. } => Some(name),
                    _ => None,
                }
            })
        {
            let krates = self
                .krates
                .adts
                .get(name)
                .map_or([].iter(), |krates| krates.iter());
            for krate in krates.filter_map(|krate| self.krates.krates.get(krate)) {
                for item in krate.impls.values() {
                    if let ItemEnum::Impl(ref impl_) = item.inner {
                        let mut generics = impl_.generics.clone();
                        generics.where_predicates.push(WherePredicate::EqPredicate {
                            lhs: types::Type::Generic("Self".to_owned()),
                            rhs: impl_.for_.clone(),
                        });

                        for item in &impl_.items {
                            if let Some(item) = krate.methods.get(item) {
                                let mut sims = query.approx(item, &generics, &mut HashMap::new());
                                // Prioritize method more than trait methods
                                if impl_.trait_.is_none() {
                                    sims.push(Similarity::Equivalent);
                                }
                                if sims.iter().any(|sim| sim != &Similarity::Different) {
                                    items_with_sims.push((item, sims))
                                }
                            }
                        }
                    }
                }
            }
        } else {
            for krate in self.krates.krates.values() {
                for item in krate.impls.values() {
                    if let ItemEnum::Impl(ref impl_) = item.inner {
                        let mut generics = impl_.generics.clone();
                        generics.where_predicates.push(WherePredicate::EqPredicate {
                            lhs: types::Type::Generic("Self".to_owned()),
                            rhs: impl_.for_.clone(),
                        });

                        for item in &impl_.items {
                            if let Some(item) = krate.methods.get(item) {
                                let mut sims = query.approx(item, &generics, &mut HashMap::new());
                                // Prioritize method more than trait methods
                                if impl_.trait_.is_none() {
                                    sims.push(Similarity::Equivalent);
                                }
                                if sims.iter().any(|sim| sim != &Similarity::Different) {
                                    items_with_sims.push((item, sims))
                                }
                            }
                        }
                    }
                }
            }
        }

        items_with_sims.sort_by_key(|(_, sims)| score(sims));

        items_with_sims
            .into_iter()
            .rev()
            .map(|(id, _)| id)
            .collect()
    }
}

fn score(sims: &[Similarity]) -> usize {
    sims.iter()
        .map(|sim| match sim {
            Similarity::Different => 0,
            Similarity::Subequal => 1,
            Similarity::Equivalent => 2,
        })
        .sum::<usize>()
        * 100
        / (2 * sims.len())
}
