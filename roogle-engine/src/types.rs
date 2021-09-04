use std::collections::HashMap;

use rustdoc_types as types;
use rustdoc_types::{Id, Item, ItemSummary};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Crates {
    pub krates: HashMap<String, Crate>,
    /// Map from a name of a ADT to names of crates containing the ADT which has the name.
    pub adts: HashMap<String, Vec<String>>,
}

impl From<Vec<types::Crate>> for Crates {
    fn from(vec: Vec<types::Crate>) -> Self {
        let mut krates: HashMap<String, Crate> = HashMap::new();
        let mut adts: HashMap<String, Vec<String>> = HashMap::new();

        for mut krate in vec {
            let name = krate
                .index
                .remove(&krate.root)
                .map(|i| i.name.unwrap())
                .unwrap();

            for ItemSummary { path, .. } in krate.paths.values() {
                adts.entry(path.last().unwrap().clone())
                    .or_default()
                    .push(name.clone());
            }
            krates.insert(name, krate.into());
        }

        Crates { krates, adts }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Crate {
    pub functions: HashMap<Id, Item>,
    pub impls: HashMap<Id, Item>,
    pub methods: HashMap<Id, Item>,
    pub paths: HashMap<Id, ItemSummary>,
}

impl From<types::Crate> for Crate {
    fn from(krate: types::Crate) -> Self {
        let types::Crate { index, paths, .. } = krate;

        let functions = index
            .clone()
            .into_iter()
            .filter(|(_, i)| matches!(i.inner, types::ItemEnum::Function(_)))
            .collect();
        let impls = index
            .clone()
            .into_iter()
            .filter(|(_, i)| matches!(i.inner, types::ItemEnum::Impl(_)))
            .collect();
        let methods = index
            .into_iter()
            .filter(|(_, i)| matches!(i.inner, types::ItemEnum::Method(_)))
            .collect();

        Crate {
            functions,
            impls,
            methods,
            paths,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Query {
    pub name: Option<Symbol>,
    pub kind: Option<QueryKind>,
}

impl Query {
    pub fn args(&self) -> Option<Vec<Argument>> {
        self.kind
            .as_ref()
            .map(|kind| {
                let QueryKind::FunctionQuery(f) = kind;
                &f.decl
            })
            .and_then(|decl| decl.inputs.clone())
    }
}

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum QueryKind {
    FunctionQuery(Function),
}

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Function {
    pub decl: FnDecl,
    // pub generics: Generics,
}

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum GenericArgs {
    AngleBracketed {
        args: Vec<Option<GenericArg>>, /* bindings: Vec<TypeBinding> */
    },
    // Parenthesized { inputs: Vec<Type>, output: Option<Type> },
}

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum GenericArg {
    // Lifetime(String),
    Type(Type),
    // Const(Constant),
}
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FnDecl {
    pub inputs: Option<Vec<Argument>>,
    pub output: Option<FnRetTy>,
    // pub c_variadic: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Argument {
    pub ty: Option<Type>,
    pub name: Option<Symbol>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FnRetTy {
    Return(Type),
    DefaultReturn,
}

pub type Symbol = String;

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Type {
    // FIXME: Give `UnresolvedPath` a better name.
    UnresolvedPath {
        name: Symbol,
        args: Option<Box<GenericArgs>>,
    },
    Generic(String),
    Primitive(PrimitiveType),
    Tuple(Vec<Option<Type>>),
    Slice(Option<Box<Type>>),
    Never,
    RawPointer {
        mutable: bool,
        type_: Box<Type>,
    },
    BorrowedRef {
        mutable: bool,
        type_: Box<Type>,
    },
}

impl Type {
    pub fn inner_type(&self) -> &Self {
        match self {
            Type::RawPointer { type_, .. } => type_.inner_type(),
            Type::BorrowedRef { type_, .. } => type_.inner_type(),
            _ => self,
        }
    }
}

/// N.B. this has to be different from `hir::PrimTy` because it also includes types that aren't
/// paths, like `Unit`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PrimitiveType {
    Isize,
    I8,
    I16,
    I32,
    I64,
    I128,
    Usize,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Char,
    Bool,
    Str,
    Unit,
    Never,
}

impl PrimitiveType {
    pub fn as_str(&self) -> &str {
        use PrimitiveType::*;
        match self {
            Isize => "isize",
            I8 => "i8",
            I16 => "i16",
            I32 => "i32",
            I64 => "i64",
            I128 => "i128",
            Usize => "usize",
            U8 => "u8",
            U16 => "u16",
            U32 => "u32",
            U64 => "u64",
            U128 => "u128",
            F32 => "f32",
            F64 => "f64",
            Char => "char",
            Bool => "bool",
            Str => "str",
            Unit => "unit",
            Never => "never",
        }
    }
}
