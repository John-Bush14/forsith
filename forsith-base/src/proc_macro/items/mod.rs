use crate::proc_macro::{Group, Ident, TokenStream, TokenTree};

pub mod parsing;
pub mod generating;

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
}

#[derive(Debug, Clone)]
pub struct GenericsDefinition(pub Vec<GenericDefinition>);

pub struct StructField {
    pub name: TokenTree,
    pub ty: TokenStream,
    pub visibility: Visibility,
    pub attributes: Vec<Attribute>,
}

pub struct StructDefinition {
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub generics: GenericsDefinition,
    pub fields: Vec<StructField>,
}

impl ItemDefinition for StructDefinition {
    fn name(&self) -> &Ident {&self.name}
    fn generics(&self) -> &GenericsDefinition {&self.generics}
}

pub struct EnumVariant {
    pub ident: Ident,
    pub fields: Option<Group>,
    pub attributes: Vec<Attribute>,
    pub discriminant: Option<TokenStream>,
}

pub struct EnumDefinition {
    pub name: Ident,
    pub generics: GenericsDefinition,
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub variants: Vec<EnumVariant>,
}

impl ItemDefinition for EnumDefinition {
    fn name(&self) -> &Ident {&self.name}
    fn generics(&self) -> &GenericsDefinition {&self.generics}
}
