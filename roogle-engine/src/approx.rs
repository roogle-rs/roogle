use std::collections::HashMap;

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
    fn approx(
        &self,
        item: &types::Item,
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
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

        sims
    }
}

impl Approximate<String> for Symbol {
    fn approx(
        &self,
        string: &String,
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        if self == string {
            vec![Equivalent]
        } else {
            vec![Different]
        }
    }
}

impl Approximate<types::ItemEnum> for QueryKind {
    fn approx(
        &self,
        kind: &types::ItemEnum,
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        use types::ItemEnum::*;
        use QueryKind::*;
        match (self, kind) {
            (FunctionQuery(q), Function(i)) => q.approx(i, generics, substs),
            _ => vec![Different],
        }
    }
}

impl Approximate<types::Function> for Function {
    fn approx(
        &self,
        function: &types::Function,
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        // update `generics` using `function.generics`
        self.decl.approx(&function.decl, generics, substs)
    }
}

impl Approximate<types::FnDecl> for FnDecl {
    fn approx(
        &self,
        decl: &types::FnDecl,
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
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
    fn approx(
        &self,
        arg: &(String, types::Type),
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
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
    fn approx(
        &self,
        ret_ty: &Option<types::Type>,
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        match (self, ret_ty) {
            (FnRetTy::Return(q), Some(i)) => q.approx(i, generics, substs),
            (FnRetTy::DefaultReturn, None) => vec![Equivalent],
            _ => vec![Different],
        }
    }
}

impl Approximate<types::Type> for Type {
    fn approx(
        &self,
        type_: &types::Type,
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        use Type::*;
        match (self, type_) {
            (Primitive(q), types::Type::Primitive(i)) => q.approx(i, generics, substs),
            (Primitive(_), _) => vec![Different],
            _ => unimplemented!(),
        }
    }
}

impl Approximate<String> for PrimitiveType {
    fn approx(
        &self,
        prim_ty: &String,
        generics: &types::Generics,
        substs: &mut HashMap<String, Type>,
    ) -> Vec<Similarity> {
        if self.as_str() == prim_ty {
            vec![Equivalent]
        } else {
            vec![Different]
        }
    }
}
