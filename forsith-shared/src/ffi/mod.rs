use std::borrow::Cow;

mod definition;

pub struct FFILib<'a> {
    items: FFIItems<'a>,
    config: FFILibConfig
}

pub struct FFILibConfig {
    pub generated_libstruct: String,
    pub header: Cow<'static, str>,
}

#[derive(Default, Debug)]
pub struct FFIItems<'a>(Vec<FFIItem<'a>>);

#[derive(Debug)]
pub enum FFIItem<'a> {
    Struct(FFIStruct<'a>),
    Function(FFIFunction<'a>),
    Mod(FFIMod<'a>),
    Use(UseItem<'a>),
    TypeAlias(TypeAlias<'a>),
    Constant(Constant<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constant<'a> {
    pub name: &'a str,
    pub ty: FFIValueType<'a>,
    pub value: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAlias<'a> {
    pub name: &'a str,
    pub ty: FFIType<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseItem<'a> {
    pub path: &'a str,
    pub visibility: Visibility<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility<'a> {
    Pub,
    Priv,
    CustomPub(&'a str)
}

#[derive(Debug)]
pub struct FFIMod<'a> {
    name: &'a str,
    items: FFIItems<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FFIStruct<'a> {
    name: &'a str,
    fields: Vec<FFIField<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FFIFunction<'a> {
    name: &'a str,
    params: Vec<FFIField<'a>>,
    ret_ty: FFIType<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FFIField<'a> {
    ty: FFIType<'a>,
    meta: FFIFieldMetadata,
    optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FFIFieldMetadata {
    None,
    Len(usize),
    LenField(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FFIType<'a> {
    value_type: FFIValueType<'a>,
    indirection: Indirection,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Indirection {
    None,
    Ptr,
    MutPtr,
    MutPtrMutPtr,
    MutPtrConstPtr,
    PtrPtr
}

