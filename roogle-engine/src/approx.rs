use std::collections::HashMap;

use log::{debug, info, trace};
use log_derive::logfn;
use rustdoc_types as types;

use crate::types::*;

pub trait Approximate<Destination> {
    fn approx(
        &self,
        dest: &Destination,
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity>;
}

trait GenericsExt {
    fn compose(&self, other: &types::Generics) -> types::Generics;
}

impl GenericsExt for types::Generics {
    fn compose(&self, other: &types::Generics) -> types::Generics {
        let mut params = self.params.clone();
        params.append(&mut other.params.clone());

        let mut where_predicates = self.where_predicates.clone();
        where_predicates.append(&mut other.where_predicates.clone());

        types::Generics {
            params,
            where_predicates,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Similarity {
    Different,
    Subequal,
    Equivalent,
}

use Similarity::*;

impl Approximate<types::Item> for Query {
    #[logfn(info, fmt = "Approximating `Query` to `Item` finished: {:?}")]
    fn approx(
        &self,
        item: &types::Item,
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        info!("-------------------------------");
        info!("Approximating `Query` to `Item`");
        trace!("approx(lhs={:?}, rhs={:?})", self, item);

        let mut sims = Vec::new();

        if let Some(ref name) = self.name {
            match item.name {
                Some(ref item_name) => sims.append(&mut name.approx(item_name, generics, substs)),
                None => sims.push(Different),
            }
        }

        if let Some(ref kind) = self.kind {
            sims.append(&mut kind.approx(&item.inner, generics, substs))
        }

        trace!("sims: {:?}", sims);
        sims
    }
}

impl Approximate<String> for Symbol {
    #[logfn(info, fmt = "Approximating `Symbol` to `String` finished: {:?}")]
    fn approx(
        &self,
        string: &String,
        _: &types::Generics,
        _: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        info!("Approximating `Symbol` to `String`");
        trace!("approx(lhs: {:?}, rhs: {:?})", self, string);

        if self == string {
            vec![Equivalent]
        } else {
            vec![Different]
        }
    }
}

impl Approximate<types::ItemEnum> for QueryKind {
    #[logfn(info, fmt = "Approximating `QueryKind` to `ItemEnum` finished: {:?}")]
    fn approx(
        &self,
        kind: &types::ItemEnum,
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        info!("Approximating `QueryKind` to `ItemEnum`");
        trace!("approx(lhs: {:?}, rhs: {:?})", self, kind);

        use types::ItemEnum::*;
        use QueryKind::*;
        match (self, kind) {
            (FunctionQuery(q), Function(i)) => q.approx(i, generics, substs),
            (FunctionQuery(q), Method(i)) => q.approx(i, generics, substs),
            _ => vec![Different],
        }
    }
}

impl Approximate<types::Function> for Function {
    #[logfn(info, fmt = "Approximating `Function` to `Function` finished: {:?}")]
    fn approx(
        &self,
        function: &types::Function,
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        info!("Approximating `Function` to `Function`");
        trace!("approx(lhs: {:?}, rhs: {:?})", self, function);

        let generics = generics.compose(&function.generics);
        self.decl.approx(&function.decl, &generics, substs)
    }
}

impl Approximate<types::Method> for Function {
    #[logfn(info, fmt = "Approximating `Function` to `Method` finished: {:?}")]
    fn approx(
        &self,
        method: &types::Method,
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        info!("Approximating `Function` to `Method`");
        trace!(
            "approx(lhs: {:?}, rhs: {:?}, generics: {:?})",
            self,
            method,
            generics
        );

        let generics = generics.compose(&method.generics);
        self.decl.approx(&method.decl, &generics, substs)
    }
}

impl Approximate<types::FnDecl> for FnDecl {
    #[logfn(info, fmt = "Approximating `FnDecl` to `FnDecl` finished: {:?}")]
    fn approx(
        &self,
        decl: &types::FnDecl,
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        info!("Approximating `FnDecl` to `FnDecl`");
        trace!("approx(lhs: {:?}, rhs: {:?})", self, decl);

        let mut sims = Vec::new();

        if let Some(ref inputs) = self.inputs {
            inputs
                .iter()
                .enumerate()
                .for_each(|(idx, input)| match decl.inputs.get(idx) {
                    Some(arg) => sims.append(&mut input.approx(arg, generics, substs)),
                    None => sims.push(Different),
                });

            if decl.inputs.len() > inputs.len() {
                let extra = decl.inputs.len() - inputs.len();
                sims.append(&mut vec![Different; extra])
            }
        }

        if let Some(ref output) = self.output {
            sims.append(&mut output.approx(&decl.output, generics, substs))
        }

        sims
    }
}

impl Approximate<(String, types::Type)> for Argument {
    #[logfn(
        info,
        fmt = "Approximating `Argument` to `(String, Type)` finished: {:?}"
    )]
    fn approx(
        &self,
        arg: &(String, types::Type),
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        info!("Approximating `Argument` to `(String, Type)`");
        trace!("approx(lhs: {:?}, rhs: {:?})", self, arg);

        let mut sims = Vec::new();

        if let Some(ref type_) = self.ty {
            sims.append(&mut type_.approx(&arg.1, generics, substs));
        }

        if let Some(ref name) = self.name {
            sims.append(&mut name.approx(&arg.0, generics, substs));
        }

        sims
    }
}

impl Approximate<Option<types::Type>> for FnRetTy {
    #[logfn(info, fmt = "Approximating `FnRetTy` to `Option<Type>` finished: {:?}")]
    fn approx(
        &self,
        ret_ty: &Option<types::Type>,
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        info!("Approximating `FnRetTy` to `Option<Type>`");
        trace!("approx(lhs: {:?}, rhs: {:?})", self, ret_ty);

        match (self, ret_ty) {
            (FnRetTy::Return(q), Some(i)) => q.approx(i, generics, substs),
            (FnRetTy::DefaultReturn, None) => vec![Equivalent],
            _ => vec![Different],
        }
    }
}

impl Approximate<types::Type> for Type {
    #[logfn(info, fmt = "Approximating `Type` to `Type` finished: {:?}")]
    fn approx(
        &self,
        type_: &types::Type,
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        info!("Approximating `Type` to `Type`");
        trace!(
            "approx(lhs: {:?}, rhs: {:?}, generics: {:?}, substs: {:?})",
            self,
            type_,
            generics,
            substs
        );

        use Type::*;
        match (self, type_) {
            (q, types::Type::Generic(i)) => {
                if i == "Self" {
                    for where_predicate in &generics.where_predicates {
                        if let types::WherePredicate::EqPredicate { lhs, rhs } = where_predicate {
                            if lhs == &types::Type::Generic("Self".to_owned()) {
                                return q.approx(rhs, generics, substs);
                            }
                        }
                    }
                }
                match substs.get(i) {
                    Some(i) => {
                        if q == i {
                            vec![Subequal]
                        } else {
                            vec![Different]
                        }
                    }
                    None => {
                        substs.insert(i.clone(), q.clone());
                        vec![Subequal]
                    }
                }
            }
            (q, types::Type::BorrowedRef { type_: i, .. }) => q.approx(i, generics, substs),
            (
                UnresolvedPath {
                    name: q,
                    args: q_args,
                },
                types::Type::ResolvedPath {
                    name: i,
                    args: i_args,
                    ..
                },
            ) => {
                let mut sims = q.approx(i, generics, substs);
                if sims == vec![Equivalent] {
                    match (q_args, i_args) {
                        (Some(q), Some(i)) => {
                            if let (
                                GenericArgs::AngleBracketed { args: q },
                                types::GenericArgs::AngleBracketed { args: i, .. },
                            ) = (&**q, &**i)
                            {
                                let q = q.iter().map(|q| match q {
                                    GenericArg::Type(q) => q,
                                });
                                let i = i.iter().filter_map(|i| match i {
                                    types::GenericArg::Type(t) => Some(t),
                                    _ => None,
                                });
                                for (q, i) in q.zip(i) {
                                    sims.append(&mut q.approx(i, generics, substs))
                                }
                            }
                        }
                        (Some(_), None) => sims.push(Different),
                        (None, _) => {}
                    }
                }
                sims
            }
            (Primitive(q), types::Type::Primitive(i)) => q.approx(i, generics, substs),
            (q, i) => {
                debug!(
                    "Potentially unimplemented approximation: approx(lhs: {:?}, rhs: {:?})",
                    q, i
                );
                vec![Different]
            }
        }
    }
}

impl Approximate<String> for PrimitiveType {
    #[logfn(
        info,
        fmt = "Approximating `PrimitiveType` to `PrimitiveType` finished: {:?}"
    )]
    fn approx(
        &self,
        prim_ty: &String,
        _: &types::Generics,
        _: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        info!("Approximating `PrimitiveType` to `String`");
        trace!("approx(lhs: {:?}, rhs: {:?})", self, prim_ty);

        if self.as_str() == prim_ty {
            vec![Equivalent]
        } else {
            vec![Different]
        }
    }
}
