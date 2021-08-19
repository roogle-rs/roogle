use std::collections::HashMap;

use log::{info, trace};
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Similarity {
    Equivalent,
    Subequal,
    Different,
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
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
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

        // update `generics` using `function.generics`
        self.decl.approx(&function.decl, generics, substs)
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
                })
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
            (q, types::Type::Generic(i)) => match substs.get(i) {
                Some(i) => {
                    if q == i {
                        vec![Equivalent]
                    } else {
                        vec![Different]
                    }
                }
                None => {
                    substs.insert(i.clone(), q.clone());
                    vec![Equivalent]
                }
            },
            (q, types::Type::BorrowedRef { type_: i, .. }) => q.approx(i, generics, substs),
            (Primitive(q), types::Type::Primitive(i)) => q.approx(i, generics, substs),
            (Primitive(_), _) => vec![Different],
            _ => unimplemented!(),
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
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
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
