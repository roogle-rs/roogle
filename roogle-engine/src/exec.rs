use std::collections::HashMap;
use std::rc::Rc;

use roogle_index::types as index;

use crate::approx::Approximate;
use crate::types::Query;

pub struct QueryExecutor {
    krate: String,
    index: index::Index,
}

impl QueryExecutor {
    pub fn new(krate: impl ToString, index: index::Index) -> Self {
        Self {
            krate: krate.to_string(),
            index,
        }
    }

    pub fn exec(&self, query: &Query) -> Vec<Rc<index::IndexItem>> {
        if let Some(krate) = self.index.crates.get(&self.krate) {
            let mut items_with_sims = Vec::new();
            for item in &krate.items {
                use index::ItemKind::*;
                match &*item.kind {
                    FunctionItem(function) => items_with_sims.push((
                        item,
                        query.approx(&item, &index::Generics::new(), &mut HashMap::new()),
                    )),
                    _ => (),
                }
            }
            items_with_sims.sort_by(|a, b| a.1.cmp(&b.1));

            items_with_sims
                .into_iter()
                .map(|(item, _)| item)
                .map(|item| Rc::clone(&item))
                .collect()
        } else {
            Vec::new()
        }
    }
}
