use std::collections::HashMap;

use rustdoc_types as types;
use serde::Serialize;
use thiserror::Error;

use crate::{
    compare::{Compare, Similarities},
    query::Query,
    Index,
};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Hit {
    pub name: String,
    pub path: Vec<String>,
    pub link: Vec<String>,
    pub docs: Option<String>,
    #[serde(skip)]
    similarities: Similarities,
}

impl Hit {
    pub fn similarities(&self) -> &Similarities {
        &self.similarities
    }
}

impl PartialOrd for Hit {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.similarities.partial_cmp(&other.similarities)
    }
}

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("crate `{0}` is not present in the index")]
    CrateNotFound(String),

    #[error("item with id `{0}` is not present in crate `{1}`")]
    ItemNotFound(String, String),
}

pub type Result<T> = std::result::Result<T, SearchError>;

/// Represents a scope to search in.
#[derive(Debug, Clone, Serialize)]
pub enum Scope {
    /// Represetns a single crate.
    Crate(String),

    /// Represents multiple crates.
    ///
    /// For example:
    /// - `rustc_ast`, `rustc_ast_lowering`, `rustc_passes` and `rustc_ast_pretty`
    /// - `std`, `core` and `alloc`
    Set(Vec<String>),
}

impl Scope {
    pub fn flatten(self) -> Vec<String> {
        match self {
            Scope::Crate(krate) => vec![krate],
            Scope::Set(krates) => krates,
        }
    }
}

impl Index {
    /// Perform search with given query and scope.
    ///
    /// Returns [`Hit`]s whose similarity score outperforms given `threshold`.
    pub fn search(&self, query: &Query, scope: Scope, threshold: f32) -> Result<Vec<Hit>> {
        let mut hits = vec![];

        let krates = scope.flatten();
        for krate_name in krates {
            let krate = self
                .crates
                .get(&krate_name)
                .ok_or_else(|| SearchError::CrateNotFound(krate_name.clone()))?;
            for item in krate.index.values() {
                match item.inner {
                    types::ItemEnum::Function(_) => {
                        let (path, link) = Self::path_and_link(krate, &krate_name, item, None)?;
                        let sims = self.compare(query, item, krate, None);

                        if sims.score() < threshold {
                            hits.push(Hit {
                                name: item.name.clone().unwrap(), // SAFETY: all functions has its name.
                                path,
                                link,
                                docs: item.docs.clone(),
                                similarities: sims,
                            });
                        }
                    }
                    types::ItemEnum::Impl(ref impl_) if impl_.trait_.is_none() => {
                        let assoc_items = impl_
                            .items
                            .iter()
                            .map(|id| {
                                krate.index.get(id).ok_or_else(|| {
                                    SearchError::ItemNotFound(id.0.clone(), krate_name.clone())
                                })
                            })
                            .collect::<Result<Vec<_>>>()?;
                        for assoc_item in assoc_items {
                            if let types::ItemEnum::Method(_) = assoc_item.inner {
                                let (path, link) = Self::path_and_link(
                                    krate,
                                    &krate_name,
                                    assoc_item,
                                    Some(impl_),
                                )?;
                                let sims = self.compare(query, assoc_item, krate, Some(impl_));

                                if sims.score() < threshold {
                                    hits.push(Hit {
                                        name: assoc_item.name.clone().unwrap(), // SAFETY: all methods has its name.
                                        path,
                                        link,
                                        docs: assoc_item.docs.clone(),
                                        similarities: sims,
                                    })
                                }
                            }
                        }
                    }
                    // TODO(hkmatsumoto): Acknowledge trait method as well.
                    _ => {}
                }
            }
        }

        hits.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        Ok(hits)
    }

    #[tracing::instrument(skip(self, krate))]
    fn compare(
        &self,
        query: &Query,
        item: &types::Item,
        krate: &types::Crate,
        impl_: Option<&types::Impl>,
    ) -> Similarities {
        let mut generics;
        if let Some(impl_) = impl_ {
            generics = impl_.generics.clone();
            generics
                .where_predicates
                .push(types::WherePredicate::EqPredicate {
                    lhs: types::Type::Generic("Self".to_owned()),
                    rhs: impl_.for_.clone(),
                });
        } else {
            generics = types::Generics::default()
        }
        let mut substs = HashMap::default();

        let sims = query.compare(item, krate, &mut generics, &mut substs);
        Similarities(sims)
    }

    /// Given `item` and optional `impl_`, compute its path and rustdoc link to `item`.
    ///
    /// `item` must be a function or a method, otherwise assertions will fail.
    fn path_and_link(
        krate: &types::Crate,
        krate_name: &str,
        item: &types::Item,
        impl_: Option<&types::Impl>,
    ) -> Result<(Vec<String>, Vec<String>)> {
        assert!(matches!(
            item.inner,
            types::ItemEnum::Function(_) | types::ItemEnum::Method(_)
        ));

        use types::Type;

        let get_path = |id: &types::Id| -> Result<Vec<String>> {
            let path = krate
                .paths
                .get(id)
                .ok_or_else(|| SearchError::ItemNotFound(id.0.clone(), krate_name.to_owned()))?
                .path
                .clone();

            Ok(path)
        };

        // If `item` is a associated item, replace the last segment of the path for the link of the ADT
        // it is binded to.
        let mut path;
        let mut link;
        if let Some(impl_) = impl_ {
            let recv;
            match (&impl_.for_, &impl_.trait_) {
                (_, Some(ref t)) => {
                    if let Type::ResolvedPath { name, id, .. } = t {
                        path = get_path(id)?;
                        recv = format!("trait.{}.html", name);
                    } else {
                        // SAFETY: All traits are represented by `ResolvedPath`.
                        unreachable!()
                    }
                }
                (
                    Type::ResolvedPath {
                        ref name, ref id, ..
                    },
                    _,
                ) => {
                    path = get_path(id)?;
                    let summary = krate.paths.get(id).ok_or_else(|| {
                        SearchError::ItemNotFound(id.0.clone(), krate_name.to_owned())
                    })?;
                    match summary.kind {
                        types::ItemKind::Union => recv = format!("union.{}.html", name),
                        types::ItemKind::Enum => recv = format!("enum.{}.html", name),
                        types::ItemKind::Struct => recv = format!("struct.{}.html", name),
                        // SAFETY: ADTs are either unions or enums or structs.
                        _ => unreachable!(),
                    }
                }
                (Type::Primitive(ref prim), _) => {
                    path = vec![prim.clone()];
                    recv = format!("primitive.{}.html", prim);
                }
                (Type::Tuple(_), _) => {
                    path = vec!["tuple".to_owned()];
                    recv = "primitive.tuple.html".to_owned();
                }
                (Type::Slice(_), _) => {
                    path = vec!["slice".to_owned()];
                    recv = "primitive.slice.html".to_owned();
                }
                (Type::Array { .. }, _) => {
                    path = vec!["array".to_owned()];
                    recv = "primitive.array.html".to_owned();
                }
                (Type::RawPointer { .. }, _) => {
                    path = vec!["pointer".to_owned()];
                    recv = "primitive.pointer.html".to_owned();
                }
                (Type::BorrowedRef { .. }, _) => {
                    path = vec!["reference".to_owned()];
                    recv = "primitive.reference.html".to_owned();
                }
                _ => unreachable!(),
            }
            link = path.clone();
            if let Some(l) = link.last_mut() {
                *l = recv;
            }
        } else {
            path = get_path(&item.id)?;
            link = path.clone();
        }

        match item.inner {
            types::ItemEnum::Function(_) => {
                if let Some(l) = link.last_mut() {
                    *l = format!("fn.{}.html", l);
                }
                Ok((path.clone(), link))
            }
            types::ItemEnum::Method(_) => {
                let name = item.name.clone().unwrap(); // SAFETY: all methods has its name.
                if let Some(l) = link.last_mut() {
                    *l = format!("{}#method.{}", l, &name);
                }
                path.push(name);

                Ok((path.clone(), link))
            }
            // SAFETY: Already asserted at the beginning of this function.
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::compare::{DiscreteSimilarity::*, Similarity::*};
    use crate::query::{FnDecl, FnRetTy, Function};

    fn krate() -> types::Crate {
        types::Crate {
            root: types::Id("0:0".to_owned()),
            crate_version: Some("0.0.0".to_owned()),
            includes_private: false,
            index: Default::default(),
            paths: Default::default(),
            external_crates: Default::default(),
            format_version: 0,
        }
    }

    fn item(name: String, inner: types::ItemEnum) -> types::Item {
        types::Item {
            id: types::Id("test".to_owned()),
            crate_id: 0,
            name: Some(name),
            span: None,
            visibility: types::Visibility::Public,
            docs: None,
            links: HashMap::default(),
            attrs: vec![],
            deprecation: None,
            inner,
        }
    }

    /// Returns a function which will be expressed as `fn foo() -> ()`.
    fn foo() -> types::Function {
        types::Function {
            decl: types::FnDecl {
                inputs: vec![],
                output: None,
                c_variadic: false,
            },
            generics: types::Generics {
                params: vec![],
                where_predicates: vec![],
            },
            header: HashSet::default(),
            abi: "rust".to_owned(),
        }
    }

    #[test]
    fn compare_symbol() {
        let query = Query {
            name: Some("foo".to_owned()),
            kind: None,
        };

        let function = foo();
        let item = item("foo".to_owned(), types::ItemEnum::Function(function));
        let krate = krate();
        let mut generics = types::Generics::default();
        let mut substs = HashMap::default();

        assert_eq!(
            query.compare(&item, &krate, &mut generics, &mut substs),
            vec![Continuous(0.0)]
        )
    }

    #[test]
    fn compare_function() {
        let q = Function {
            decl: FnDecl {
                inputs: Some(vec![]),
                output: Some(FnRetTy::DefaultReturn),
            },
        };

        let i = foo();

        let krate = krate();
        let mut generics = types::Generics::default();
        let mut substs = HashMap::default();

        assert_eq!(
            q.compare(&i, &krate, &mut generics, &mut substs),
            vec![Discrete(Equivalent), Discrete(Equivalent)]
        )
    }
}
