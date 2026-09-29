use std::borrow::Cow;

use forsith_base::proc_macro::items::Indirection;

// mod definition;

pub mod generation;

pub struct FFIGenerator {
    pub func_map: FFiFuncMap,
    pub struct_map: FFIStructMap,
}

pub struct FFiFuncMap {
    pub loader: fn(&str) -> Cow<&str>,
    pub wrapper: fn(&str) -> Cow<&str>,
    pub ty: fn(&str) -> Cow<&str>,
    pub raw: fn(&str) -> Cow<&str>,
}

pub struct FFIStructMap {
    pub raw: fn(&str) -> Cow<&str>,
    pub wrapper: fn(&str) -> Cow<&str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FFIStructDef<'a> {
    pub name: &'a str,
    pub fields: Vec<FFIField<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FFIFuncDef<'a> {
    pub name: &'a str,
    pub params: Vec<FFIField<'a>>,
    pub ret_ty: Option<FFIType<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FFIField<'a> {
    pub name: &'a str,
    pub ty: FFIValueType<'a>,
    pub meta: Vec<FFiFieldIndirection<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FFiFieldIndirection<'a> {
    mutable: bool,
    optional: bool,
    len: IndirectionLen<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndirectionLen<'a> {
    None,
    Fixed(usize),
    Field(&'a str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FFIType<'a> {
    pub value_type: FFIValueType<'a>,
    pub indirection: Vec<Indirection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FFIValueType<'a> {
    Void,
    Primitive(FFIPrimitive),
    Custom(&'a str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FFIPrimitive {
    Bool,
    Char,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
}
