use std::collections::HashMap;

use rustdoc_types::*;

use crate::approx::Approximate;
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
                    let mut sims = query.approx(item, &Generics::default(), &mut HashMap::new());
                    sims.sort();

                    items_with_sims.push((&item.id, sims))
                }
                _ => (),
            }
        }
        items_with_sims.sort_by(|(_, a), (_, b)| a.cmp(b));

        items_with_sims
            .into_iter()
            .map(|(id, _)| id)
            .map(|id| self.krate.index.get(id).unwrap())
            .collect()
    }
}
