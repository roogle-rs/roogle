use std::collections::HashMap;
use std::rc::Rc;

use serde::Deserialize;

type Symbol = String;

#[non_exhaustive]
#[derive(Deserialize, Clone, Debug)]
pub struct Index {
    #[serde(flatten)]
    pub crates: HashMap<String, CrateData>,
}

// Lines below are copied & adjusted from librustdoc.

#[derive(Deserialize, Clone, Debug)]
pub struct CrateData {
    pub items: Vec<Rc<IndexItem>>,
}

/// Struct representing one entry in the JS search index. These are all emitted
/// by hand to a large JS file at the end of cache-creation.
#[derive(Deserialize, Clone, Debug)]
pub struct IndexItem {
    pub name: String,
    pub kind: Box<ItemKind>,
    pub path: String,
    pub desc: String,
}

/// Anything with a source location and set of attributes and, optionally, a
/// name. That is, anything that can be documented. This doesn't correspond
/// directly to the AST's concept of an item; it's a strict superset.
#[derive(Deserialize, Clone, Debug)]
pub struct Item {
    /// The name of this item.
    /// Optional because not every item has a name, e.g. impls.
    pub name: Option<Symbol>,
    pub visibility: Visibility,
    /// Information about this item that is specific to what kind of item it is.
    /// E.g., struct vs enum vs function.
    pub kind: Box<ItemKind>,
}

#[derive(Deserialize, Clone, Debug)]
pub enum ItemKind {
    ExternCrateItem {
        /// The crate's name, *not* the name it's imported as.
        #[serde(skip)]
        src: Option<Symbol>,
    },
    ImportItem(Import),
    StructItem(Struct),
    UnionItem(Union),
    EnumItem(Enum),
    FunctionItem(Function),
    ModuleItem(Module),
    TypedefItem(Typedef, bool /* is associated type */),
    OpaqueTyItem(OpaqueTy),
    StaticItem(Static),
    ConstantItem(Constant),
    TraitItem(Trait),
    TraitAliasItem(TraitAlias),
    ImplItem(Impl),
    /// A method signature only. Used for required methods in traits (ie,
    /// non-default-methods).
    TyMethodItem(Function),
    /// A method with a body.
    // HACK(hkamtsumoto): `The second field was originally `rustc_hir::Defaultness`, which is hard
    // to implement `Deserialize`. As a workaround, we skip deserializing it and use a fake unit
    // type instead.
    MethodItem(Function, #[serde(skip)] Option<()>),
    StructFieldItem(Type),
    VariantItem(Variant),
    /// `fn`s from an extern block
    ForeignFunctionItem(Function),
    /// `static`s from an extern block
    ForeignStaticItem(Static),
    /// `type`s from an extern block
    ForeignTypeItem,
    MacroItem(Macro),
    PrimitiveItem(PrimitiveType),
    AssocConstItem(Type, Option<String>),
    /// An associated item in a trait or trait impl.
    ///
    /// The bounds may be non-empty if there is a `where` clause.
    /// The `Option<Type>` is the default concrete type (e.g. `trait Trait { type Target = usize; }`)
    AssocTypeItem(Vec<GenericBound>, Option<Type>),
    /// An item that has been stripped by a rustdoc pass
    StrippedItem(Box<ItemKind>),
    KeywordItem(Symbol),
}

#[derive(Deserialize, Clone, Debug)]
pub struct Module {
    pub items: Vec<Item>,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub enum GenericBound {
    TraitBound(PolyTrait, #[serde(skip)] ()),
    Outlives(Lifetime),
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Lifetime(pub Symbol);

#[derive(Deserialize, Clone, Debug)]
pub enum WherePredicate {
    BoundPredicate {
        ty: Type,
        bounds: Vec<GenericBound>,
        bound_params: Vec<Lifetime>,
    },
    RegionPredicate {
        lifetime: Lifetime,
        bounds: Vec<GenericBound>,
    },
    EqPredicate {
        lhs: Type,
        rhs: Type,
    },
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub enum GenericParamDefKind {
    Lifetime,
    Type {
        bounds: Vec<GenericBound>,
        default: Option<Type>,
    },
    Const {
        ty: Type,
        default: Option<String>,
    },
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub struct GenericParamDef {
    pub name: Symbol,
    pub kind: GenericParamDefKind,
}

// maybe use a Generic enum and use Vec<Generic>?
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Generics {
    pub params: Vec<GenericParamDef>,
    pub where_predicates: Vec<WherePredicate>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Function {
    pub decl: FnDecl,
    pub generics: Generics,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub struct FnDecl {
    pub inputs: Arguments,
    pub output: FnRetTy,
    pub c_variadic: bool,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Arguments {
    pub values: Vec<Argument>,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Argument {
    pub type_: Type,
    pub name: Symbol,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Mutability {
    Mut,
    Not,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub enum SelfTy {
    SelfValue,
    SelfBorrowed(Option<Lifetime>, Mutability),
    SelfExplicit(Type),
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub enum FnRetTy {
    Return(Type),
    DefaultReturn,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Trait {
    pub items: Vec<Item>,
    pub generics: Generics,
    pub bounds: Vec<GenericBound>,
    pub is_auto: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TraitAlias {
    pub generics: Generics,
    pub bounds: Vec<GenericBound>,
}

/// A trait reference, which may have higher ranked lifetimes.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub struct PolyTrait {
    pub trait_: Type,
    pub generic_params: Vec<GenericParamDef>,
}

/// A representation of a type suitable for hyperlinking purposes. Ideally, one can get the original
/// type out of the AST/`TyCtxt` given one of these, if more information is needed. Most
/// importantly, it does not preserve mutability or boxes.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Type {
    /// Structs/enums/traits (most that would be an `hir::TyKind::Path`).
    ResolvedPath {
        path: Path,
        /// `true` if is a `T::Name` path for associated types.
        is_generic: bool,
    },
    /// `dyn for<'a> Trait<'a> + Send + 'static`
    DynTrait(Vec<PolyTrait>, Option<Lifetime>),
    /// For parameterized types, so the consumer of the JSON don't go
    /// looking for types which don't exist anywhere.
    Generic(Symbol),
    /// Primitives are the fixed-size numeric types (plus int/usize/float), char,
    /// arrays, slices, and tuples.
    Primitive(PrimitiveType),
    /// `extern "ABI" fn`
    BareFunction(Box<BareFunctionDecl>),
    Tuple(Vec<Type>),
    Slice(Box<Type>),
    /// The `String` field is about the size or the constant representing the array's length.
    Array(Box<Type>, String),
    Never,
    RawPointer(Mutability, Box<Type>),
    BorrowedRef {
        lifetime: Option<Lifetime>,
        mutability: Mutability,
        type_: Box<Type>,
    },

    // `<Type as Trait>::Name`
    QPath {
        name: Symbol,
        self_type: Box<Type>,
        trait_: Box<Type>,
    },

    // `_`
    Infer,

    // `impl TraitA + TraitB + ...`
    ImplTrait(Vec<GenericBound>),
}

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Copy, Debug)]
/// N.B. this has to be different from `hir::PrimTy` because it also includes types that aren't
/// paths, like `Unit`.
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

#[derive(Deserialize, Copy, Clone, Debug)]
pub enum Visibility {
    /// `pub`
    Public,
    /// Visibility inherited from parent.
    ///
    /// For example, this is the visibility of private items and of enum variants.
    Inherited,
    /// `pub(crate)`, `pub(super)`, or `pub(in path::to::somewhere)`
    Restricted(#[serde(skip)] ()),
}

#[derive(Deserialize, Clone, Debug)]
pub struct Struct {
    pub generics: Generics,
    pub fields: Vec<Item>,
    pub fields_stripped: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Union {
    pub generics: Generics,
    pub fields: Vec<Item>,
    pub fields_stripped: bool,
}

/// This is a more limited form of the standard Struct, different in that
/// it lacks the things most items have (name, id, parameterization). Found
/// only as a variant in an enum.
#[derive(Deserialize, Clone, Debug)]
pub struct VariantStruct {
    pub fields: Vec<Item>,
    pub fields_stripped: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Enum {
    #[serde(skip)]
    pub variants: Vec<Item>,
    pub generics: Generics,
    pub variants_stripped: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub enum Variant {
    CLike,
    Tuple(Vec<Type>),
    Struct(VariantStruct),
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Path {
    pub global: bool,
    pub segments: Vec<PathSegment>,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub enum GenericArg {
    Lifetime(Lifetime),
    Type(Type),
    Const(Constant),
    Infer,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub enum GenericArgs {
    AngleBracketed {
        args: Vec<GenericArg>,
        bindings: Vec<TypeBinding>,
    },
    Parenthesized {
        inputs: Vec<Type>,
        output: Option<Type>,
    },
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub struct PathSegment {
    pub name: Symbol,
    pub args: GenericArgs,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Typedef {
    pub type_: Type,
    pub generics: Generics,
    /// `type_` can come from either the HIR or from metadata. If it comes from HIR, it may be a type
    /// alias instead of the final type. This will always have the final type, regardless of whether
    /// `type_` came from HIR or from metadata.
    ///
    /// If `item_type.is_none()`, `type_` is guarenteed to come from metadata (and therefore hold the
    /// final type).
    pub item_type: Option<Type>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct OpaqueTy {
    pub bounds: Vec<GenericBound>,
    pub generics: Generics,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub struct BareFunctionDecl {
    pub generic_params: Vec<GenericParamDef>,
    pub decl: FnDecl,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Static {
    pub type_: Type,
    pub mutability: Mutability,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Constant {
    pub type_: Type,
    pub kind: ConstantKind,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ConstantKind {
    /// This is the wrapper around `ty::Const` for a non-local constant. Because it doesn't have a
    /// `BodyId`, we need to handle it on its own.
    ///
    /// Note that `ty::Const` includes generic parameters, and may not always be uniquely identified
    /// by a DefId. So this field must be different from `Extern`.
    TyConst { expr: String },
    /// A constant (expression) that's not an item or associated item. These are usually found
    /// nested inside types (e.g., array lengths) or expressions (e.g., repeat counts), and also
    /// used to define explicit discriminant values for enum variants.
    Anonymous {
        #[serde(skip)]
        def_id: (),
    },
    /// A constant from a different crate.
    Extern {
        #[serde(skip)]
        def_id: (),
    },
    /// `const FOO: u32 = ...;`
    Local {
        #[serde(skip)]
        def_id: (),
        #[serde(skip)]
        body: (),
    },
}

#[derive(Deserialize, Clone, Debug)]
pub struct Impl {
    pub generics: Generics,
    pub trait_: Option<Type>,
    pub for_: Type,
    pub items: Vec<Item>,
    pub negative_polarity: bool,
    pub synthetic: bool,
    pub blanket_impl: Option<Box<Type>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Import {
    pub kind: ImportKind,
    pub source: ImportSource,
    pub should_be_displayed: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub enum ImportKind {
    // use source as str;
    Simple(Symbol),
    // use source::*;
    Glob,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ImportSource {
    pub path: Path,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Macro {
    pub source: String,
    #[serde(skip)]
    pub imported_from: Option<Symbol>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ProcMacro {
    #[serde(skip)]
    pub helpers: Vec<Symbol>,
}

/// An type binding on an associated type (e.g., `A = Bar` in `Foo<A = Bar>` or
/// `A: Send + Sync` in `Foo<A: Send + Sync>`).
#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub struct TypeBinding {
    pub name: Symbol,
    pub kind: TypeBindingKind,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub enum TypeBindingKind {
    Equality { ty: Type },
    Constraint { bounds: Vec<GenericBound> },
}
