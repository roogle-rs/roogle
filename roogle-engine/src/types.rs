use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Query {
    pub name: Option<Symbol>,
    pub kind: Option<QueryKind>,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize)]
pub enum QueryKind {
    FunctionQuery(Function),
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize)]
pub struct Function {
    pub decl: FnDecl,
    // pub generics: Generics,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize)]
pub struct FnDecl {
    pub inputs: Option<Vec<Argument>>,
    pub output: Option<FnRetTy>,
    // pub c_variadic: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Argument {
    pub ty: Option<Type>,
    pub name: Option<Symbol>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FnRetTy {
    Return(Type),
    DefaultReturn,
}

pub type Symbol = String;

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize)]
pub enum Type {
    Primitive(PrimitiveType),
}

/// N.B. this has to be different from `hir::PrimTy` because it also includes types that aren't
/// paths, like `Unit`.
#[derive(Debug, Serialize, Deserialize)]
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
    Slice,
    Array,
    Tuple,
    Unit,
    RawPointer,
    Reference,
    Fn,
    Never,
}
