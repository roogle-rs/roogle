use roogle_index::types as index;

use crate::types::*;

pub trait Approximate {
    type Item;

    fn approx(&self, item: &Self::Item) -> Vec<Similarity>;
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Similarity {
    Equivalent,
    Subequal,
    Different,
}

use Similarity::*;

impl Approximate for Query {
    type Item = index::IndexItem;

    fn approx(&self, item: &Self::Item) -> Vec<Similarity> {
        let mut sims = Vec::new();

        if let Some(ref name) = self.name {
            sims.append(&mut name.approx(&item.name))
        }

        if let Some(ref kind) = self.kind {
            sims.append(&mut kind.approx(&item.kind))
        }

        sims
    }
}

impl Approximate for String {
    type Item = String;

    fn approx(&self, item: &Self::Item) -> Vec<Similarity> {
        if self == item {
            vec![Equivalent]
        } else {
            vec![Different]
        }
    }
}

impl Approximate for QueryKind {
    type Item = index::ItemKind;

    fn approx(&self, kind: &Self::Item) -> Vec<Similarity> {
        use index::ItemKind::*;
        use QueryKind::*;
        match (self, kind) {
            (FunctionQuery(fq), FunctionItem(fi)) => fq.approx(fi),
            _ => vec![Different],
        }
    }
}

impl Approximate for Function {
    type Item = index::Function;

    fn approx(&self, function: &Self::Item) -> Vec<Similarity> {
        self.decl.approx(&function.decl)
    }
}

impl Approximate for FnDecl {
    type Item = index::FnDecl;

    fn approx(&self, decl: &Self::Item) -> Vec<Similarity> {
        let mut sims = Vec::new();

        if let Some(ref inputs) = self.inputs {
            inputs
                .iter()
                .enumerate()
                .for_each(|(idx, input)| match input {
                    Some(input) => sims.append(&mut input.approx(&decl.inputs.values[idx])),
                    None => sims.push(Different),
                })
        }

        if let Some(ref output) = self.output {
            sims.append(&mut output.approx(&decl.output))
        }

        sims
    }
}

impl Approximate for Argument {
    type Item = index::Argument;

    fn approx(&self, item: &Self::Item) -> Vec<Similarity> {
        let mut sims = Vec::new();

        if let Some(ref type_) = self.ty {
            sims.append(&mut type_.approx(&item.type_))
        }

        if let Some(ref name) = self.name {
            sims.append(&mut name.approx(&item.name))
        }

        sims
    }
}

impl Approximate for FnRetTy {
    type Item = index::FnRetTy;

    fn approx(&self, ret: &Self::Item) -> Vec<Similarity> {
        match (self, ret) {
            (FnRetTy::Return(tq), index::FnRetTy::Return(ti)) => tq.approx(ti),
            (FnRetTy::DefaultReturn, index::FnRetTy::DefaultReturn) => vec![Equivalent],
            _ => vec![Different],
        }
    }
}

impl Approximate for Type {
    type Item = index::Type;

    fn approx(&self, type_: &Self::Item) -> Vec<Similarity> {
        match (self, type_) {
            (Type::Primitive(ptq), index::Type::Primitive(pqi)) => ptq.approx(pqi),
            _ => unimplemented!(),
        }
    }
}

impl Approximate for PrimitiveType {
    type Item = index::PrimitiveType;

    fn approx(&self, pt: &Self::Item) -> Vec<Similarity> {
        use PrimitiveType::*;
        // TODO(hkmatsumoto): Do this more elegantly with macro.
        match (self, pt) {
            (Isize, index::PrimitiveType::Isize) => vec![Equivalent],
            (Usize, index::PrimitiveType::Usize) => vec![Equivalent],
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
