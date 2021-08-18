use std::collections::HashMap;
use std::rc::Rc;

use roogle_index::types as index;

use crate::types::*;

pub trait Approximate<Destination> {
    fn approx(
        &self,
        dest: &Destination,
        generics: &index::Generics,
        substs: &mut HashMap<Symbol, Type>,
    ) -> Vec<Similarity>;
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Similarity {
    Equivalent,
    Subequal,
    Different,
}

use Similarity::*;

impl Approximate<index::IndexItem> for Query {
    fn approx(
        &self,
        item: &index::IndexItem,
        generics: &index::Generics,
        substs: &mut HashMap<Symbol, Type>,
    ) -> Vec<Similarity> {
        let mut sims = Vec::new();

        if let Some(ref name) = self.name {
            sims.append(&mut name.approx(&item.name, generics, substs))
        }

        if let Some(ref kind) = self.kind {
            sims.append(&mut kind.approx(&item.kind, generics, substs))
        }

        sims
    }
}

impl Approximate<index::Symbol> for Symbol {
    fn approx(
        &self,
        symbol: &index::Symbol,
        generics: &index::Generics,
        substs: &mut HashMap<Symbol, Type>,
    ) -> Vec<Similarity> {
        if self == symbol {
            vec![Equivalent]
        } else {
            vec![Different]
        }
    }
}

impl Approximate<index::ItemKind> for QueryKind {
    fn approx(
        &self,
        kind: &index::ItemKind,
        generics: &index::Generics,
        substs: &mut HashMap<Symbol, Type>,
    ) -> Vec<Similarity> {
        use index::ItemKind::*;
        use QueryKind::*;
        match (self, kind) {
            (FunctionQuery(fq), FunctionItem(fi)) => fq.approx(fi, generics, substs),
            _ => vec![Different],
        }
    }
}

impl Approximate<index::Function> for Function {
    fn approx(
        &self,
        function: &index::Function,
        generics: &index::Generics,
        substs: &mut HashMap<Symbol, Type>,
    ) -> Vec<Similarity> {
        let generics = generics.compose(&function.generics);

        self.decl.approx(&function.decl, &generics, substs)
    }
}

impl Approximate<index::FnDecl> for FnDecl {
    fn approx(
        &self,
        decl: &index::FnDecl,
        generics: &index::Generics,
        substs: &mut HashMap<Symbol, Type>,
    ) -> Vec<Similarity> {
        let mut sims = Vec::new();

        if let Some(ref inputs) = self.inputs {
            inputs
                .iter()
                .enumerate()
                .for_each(|(idx, input)| match decl.inputs.values.get(idx) {
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

impl Approximate<index::Argument> for Argument {
    fn approx(
        &self,
        arg: &index::Argument,
        generics: &index::Generics,
        substs: &mut HashMap<Symbol, Type>,
    ) -> Vec<Similarity> {
        let mut sims = Vec::new();

        if let Some(ref type_) = self.ty {
            sims.append(&mut type_.approx(&arg.type_, generics, substs))
        }

        if let Some(ref name) = self.name {
            sims.append(&mut name.approx(&arg.name, generics, substs))
        }

        sims
    }
}

impl Approximate<index::FnRetTy> for FnRetTy {
    fn approx(
        &self,
        ret: &index::FnRetTy,
        generics: &index::Generics,
        substs: &mut HashMap<Symbol, Type>,
    ) -> Vec<Similarity> {
        match (self, ret) {
            (FnRetTy::Return(q), index::FnRetTy::Return(i)) => q.approx(i, generics, substs),
            (FnRetTy::DefaultReturn, index::FnRetTy::DefaultReturn) => vec![Equivalent],
            _ => vec![Different],
        }
    }
}

impl Approximate<index::Type> for Type {
    fn approx(
        &self,
        type_: &index::Type,
        generics: &index::Generics,
        substs: &mut HashMap<Symbol, Type>,
    ) -> Vec<Similarity> {
        use Type::*;
        match (self, type_) {
            (Primitive(q), index::Type::Primitive(i)) => q.approx(i, generics, substs),
            (Primitive(_), _) => vec![Different],
            _ => unimplemented!(),
        }
    }
}

impl Approximate<index::PrimitiveType> for PrimitiveType {
    fn approx(
        &self,
        prim_ty: &index::PrimitiveType,
        generics: &index::Generics,
        substs: &mut HashMap<Symbol, Type>,
    ) -> Vec<Similarity> {
        use PrimitiveType::*;
        match (self, prim_ty) {
            (Isize, index::PrimitiveType::Isize) | (Usize, index::PrimitiveType::Usize) => {
                vec![Equivalent]
            }
            _ => vec![Different],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `fn f(a: usize) -> ()`
    fn index_item() -> index::IndexItem {
        index::IndexItem {
            name: "f".to_owned(),
            kind: Box::new(index::ItemKind::FunctionItem(index::Function {
                decl: index::FnDecl {
                    inputs: index::Arguments {
                        values: vec![index::Argument {
                            type_: index::Type::Primitive(index::PrimitiveType::Usize),
                            name: "a".to_owned(),
                        }],
                    },
                    output: index::FnRetTy::DefaultReturn,
                    c_variadic: false,
                },
                generics: index::Generics {
                    params: vec![],
                    where_predicates: vec![],
                },
            })),
            path: "".to_owned(),
            desc: "".to_owned(),
        }
    }

    /// `fn(a: usize) -> usize`
    fn index_function() -> index::Function {
        index::Function {
            decl: index::FnDecl {
                inputs: index::Arguments {
                    values: vec![index::Argument {
                        type_: index::Type::Primitive(index::PrimitiveType::Usize),
                        name: "a".to_owned(),
                    }],
                },
                output: index::FnRetTy::Return(index::Type::Primitive(index::PrimitiveType::Usize)),
                c_variadic: false,
            },
            generics: index::Generics {
                params: vec![],
                where_predicates: vec![],
            },
        }
    }

    #[test]
    fn test_query() {
        let index_item = index_item();

        let q = Query {
            name: None,
            kind: None,
        };
        assert_eq!(q.approx(&index_item), vec![]);

        let q = Query {
            name: Some("foo".to_owned()),
            kind: None,
        };
        assert_eq!(q.approx(&index_item), vec![Different]);

        let q = Query {
            name: Some("f".to_owned()),
            kind: None,
        };
        assert_eq!(q.approx(&index_item), vec![Equivalent]);
    }

    #[test]
    fn test_function_query() {
        // `fn(a: usize) -> usize`
        let index_function = index_function();

        // `fn(_) -> _`
        let f = Function {
            decl: FnDecl {
                inputs: None,
                output: None,
            },
        };
        assert_eq!(f.approx(&index_function), vec![]);

        // `fn(a: _) -> _`
        let f = Function {
            decl: FnDecl {
                inputs: Some(vec![Some(Argument {
                    ty: None,
                    name: Some("a".to_owned()),
                })]),
                output: None,
            },
        };
        assert_eq!(f.approx(&index_function), vec![Equivalent]);

        // `fn(a: usize) -> _`
        let f = Function {
            decl: FnDecl {
                inputs: Some(vec![Some(Argument {
                    ty: Some(Type::Primitive(PrimitiveType::Usize)),
                    name: Some("a".to_owned()),
                })]),
                output: None,
            },
        };
        assert_eq!(f.approx(&index_function), vec![Equivalent, Equivalent]);

        // `fn(a: isize) -> _`
        let f = Function {
            decl: FnDecl {
                inputs: Some(vec![Some(Argument {
                    ty: Some(Type::Primitive(PrimitiveType::Isize)),
                    name: Some("a".to_owned()),
                })]),
                output: None,
            },
        };
        assert_eq!(f.approx(&index_function), vec![Different, Equivalent]);

        // `fn(a: usize) -> usize`
        let f = Function {
            decl: FnDecl {
                inputs: Some(vec![Some(Argument {
                    ty: Some(Type::Primitive(PrimitiveType::Usize)),
                    name: Some("a".to_owned()),
                })]),
                output: Some(FnRetTy::Return(Type::Primitive(PrimitiveType::Usize))),
            },
        };
        assert_eq!(
            f.approx(&index_function),
            vec![Equivalent, Equivalent, Equivalent]
        );
    }
}
