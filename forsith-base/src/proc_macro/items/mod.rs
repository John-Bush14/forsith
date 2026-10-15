use crate::alloc::vec::Vec;
use crate::proc_macro::{Group, Ident, TokenStream, TokenTree};

pub mod generating;
pub mod parsing;

pub mod struct_definition;

pub mod enum_definition;

#[derive(Debug, Clone)]
pub enum ItemContent {
    Struct(Vec<(TokenTree, TokenStream, Vec<Attribute>)>),
    Enum(Vec<(Ident, Option<Group>, Vec<Attribute>)>),
}

#[derive(Debug, Clone)]
pub enum GenericDefinition {
    Lifetime(Ident),
    Type(Ident, TokenStream),
}

#[derive(Debug, Clone)]
pub enum Visibility {
    Public,
    SpecializedPublic(TokenStream),
    Private,
}

#[derive(Debug, Clone)]
pub struct ItemPrefix {
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub constness: bool,
    pub unsafety: bool,
    pub asyncness: bool,
    pub name: Ident,
    pub generics: GenericsDefinition,
    pub item_type: ItemType,
}

#[derive(Debug, Clone)]
pub enum ItemType {
    Struct,
    Enum,
    Function,
    Trait,
    Impl,
    Mod,
    Use,
    Const,
    Static,
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: Ident,
    pub args: Option<Group>,
}

pub trait ItemDefinition {
    fn name(&self) -> &Ident;
    fn generics(&self) -> &GenericsDefinition;
    fn attributes(&self) -> &[Attribute];
}

#[derive(Debug, Clone, Default)]
pub struct GenericsDefinition(pub Vec<GenericDefinition>);

pub struct TypeAliasDefinition {
    pub name: Ident,
    pub generics: GenericsDefinition,
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub ty: TokenStream,
}

impl ItemDefinition for TypeAliasDefinition {
    fn name(&self) -> &Ident {
        &self.name
    }
    fn generics(&self) -> &GenericsDefinition {
        &self.generics
    }
    fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Indirection {
    None,
    Constant,
    Mutable,
}

pub struct FunctionParam {
    pub pattern: TokenStream,
    pub ty: TokenStream,
}

pub struct FunctionDefinition {
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub constness: bool,
    pub unsafety: bool,
    pub asyncness: bool,
    pub name: Ident,
    pub self_param: Option<Indirection>,
    pub generics: GenericsDefinition,
    pub params: Vec<FunctionParam>,
    pub ret_ty: Option<TokenStream>,
    pub body: Option<TokenStream>,
}

impl ItemDefinition for FunctionDefinition {
    fn name(&self) -> &Ident {
        &self.name
    }
    fn generics(&self) -> &GenericsDefinition {
        &self.generics
    }
    fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }
}
