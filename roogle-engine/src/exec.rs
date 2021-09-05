use std::collections::HashMap;

use rustdoc_types as types;
use rustdoc_types::{Generics, ItemEnum, WherePredicate};

use crate::approx::{Approximate, Similarity};
use crate::types::{Crates, Item, Query, Type};

pub struct QueryExecutor {
    krates: Crates,
}

impl QueryExecutor {
    pub fn new(krates: Crates) -> Self {
        Self { krates }
    }

    pub fn exec(&self, query: Query) -> Vec<Item> {
        let mut items_with_sims = Vec::new();
        for krate in self.krates.krates.values() {
            for function in krate.functions.values() {
                let sims = query.approx(function, &Generics::default(), &mut HashMap::new());
                if sims.iter().any(|sim| sim != &Similarity::Different) {
                    let mut link = krate.paths.get(&function.id).unwrap().path.clone();
                    if let Some(last) = link.last_mut() {
                        *last = format!("fn.{}.html", last);
                    }

                    let item = Item {
                        path: krate.paths.get(&function.id).unwrap().path.clone(),
                        link,
                        docs: function.docs.clone(),
                    };
                    items_with_sims.push((item, sims))
                }
            }
        }

        let krates: Vec<_>;
        if let Some(name) = query
            .args()
            .as_ref()
            .and_then(|args| args.first())
            .and_then(|arg| arg.ty.as_ref())
            .and_then(|ty| match ty.inner_type() {
                Type::UnresolvedPath { name, .. } => Some(name),
                _ => None,
            })
        {
            krates = self
                .krates
                .adts
                .get(name)
                .map_or([].iter(), |krates| krates.iter())
                .filter_map(|krate| self.krates.krates.get(krate))
                .collect();
        } else {
            krates = self.krates.krates.values().into_iter().collect();
        };

        for krate in krates {
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

                            let last;
                            if sims.iter().any(|sim| sim != &Similarity::Different) {
                                let mut path = if let Some(ref t) = impl_.trait_ {
                                    if let types::Type::ResolvedPath { name, id, .. } = t {
                                        last = format!("trait.{}.html", name);
                                        krate.paths.get(&id).unwrap().path.clone()
                                    } else {
                                        unreachable!()
                                    }
                                } else {
                                    match impl_.for_ {
                                        types::Type::ResolvedPath { ref id, .. } => {
                                            let summary = krate.paths.get(id).unwrap();
                                            let name = summary.path.last().unwrap();
                                            last = match summary.kind {
                                                types::ItemKind::Enum => {
                                                    format!("enum.{}.html", name)
                                                }
                                                types::ItemKind::Struct => {
                                                    format!("struct.{}.html", name)
                                                }
                                                _ => unreachable!(),
                                            };
                                            krate.paths.get(&id).unwrap().path.clone()
                                        }
                                        types::Type::Primitive(ref prim) => {
                                            last = format!("primitive.{}.html", prim);
                                            vec![prim.clone()]
                                        }
                                        _ => unreachable!(),
                                    }
                                };
                                let mut link = path.clone();
                                path.push(item.name.clone().unwrap());

                                if let Some(l) = link.last_mut() {
                                    *l = last;
                                }

                                if let types::ItemEnum::Method(types::Method { has_body, .. }) =
                                    item.inner
                                {
                                    if impl_.trait_.is_none() || has_body {
                                        link.last_mut().into_iter().for_each(|l| {
                                            l.push_str(&format!(
                                                "#method.{}",
                                                item.name.clone().unwrap()
                                            ))
                                        });
                                    } else {
                                        link.last_mut().into_iter().for_each(|l| {
                                            l.push_str(&format!(
                                                "#tymethod.{}",
                                                item.name.clone().unwrap()
                                            ))
                                        })
                                    }
                                }

                                let item = Item {
                                    path,
                                    link,
                                    docs: item.docs.clone(),
                                };
                                items_with_sims.push((item, sims))
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
