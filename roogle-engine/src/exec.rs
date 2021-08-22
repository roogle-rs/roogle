use std::collections::HashMap;

use rustdoc_types::*;

use crate::approx::{Approximate, Similarity};
use crate::types::Query;

pub struct QueryExecutor {
    krate: Crate,
}

impl QueryExecutor {
    pub fn new(krate: Crate) -> Self {
        Self { krate }
    }

    pub fn exec(&self, query: Query) -> Vec<&Item> {
        let mut items_with_sims = Vec::new();
        for item in self.krate.index.values() {
            match item.inner {
                ItemEnum::Function(_) => {
                    let sims = query.approx(item, &Generics::default(), &mut HashMap::new());
                    if sims.iter().any(|sim| sim != &Similarity::Different) {
                        items_with_sims.push((&item.id, sims))
                    }
                }
                ItemEnum::Impl(ref impl_) => {
                    let mut generics = Generics::default();
                    generics.where_predicates.push(WherePredicate::EqPredicate {
                        lhs: Type::Generic("Self".to_owned()),
                        rhs: impl_.for_.clone(),
                    });

                    for item in &impl_.items {
                        let item = self.krate.index.get(item).unwrap();
                        let sims = query.approx(item, &generics, &mut HashMap::new());
                        if sims.iter().any(|sim| sim != &Similarity::Different) {
                            items_with_sims.push((&item.id, sims))
                        }
                    }
                }
                _ => (),
            }
        }
        items_with_sims.sort_by_key(|(_, sims)| score(sims));

        items_with_sims
            .into_iter()
            .rev()
            .map(|(id, _)| self.krate.index.get(id).unwrap())
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
