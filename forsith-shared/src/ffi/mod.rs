use std::borrow::Cow;

use crate::interner::{InternedString, StringInterner};

mod definition;
pub use definition::InternedFFISymbols;

pub struct FFILib {
    items: FFIItems,
    config: FFILibConfig,
}

pub struct FFILibConfig {
    pub generated_libstruct: String,
    pub header: Cow<'static, str>,
}

#[derive(Default, Debug)]
pub struct FFIItems(Vec<FFIItem>);

#[derive(Debug)]
pub enum FFIItem {
    Struct(FFIStruct),
    Function(FFIFunction),
    Mod(FFIMod),
    Use(UseItem),
    TypeAlias(TypeAlias),
    Constant(Constant),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constant {
    pub name: InternedString,
    pub ty: FFIValueType,
    pub value: InternedString,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAlias {
    pub name: InternedString,
    pub ty: FFIType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseItem {
    pub path: InternedString,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    Pub,
    Priv,
    CustomPub(InternedString)
}

#[derive(Debug)]
pub struct FFIMod {
    name: InternedString,
    items: FFIItems,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FFIStruct {
    name: InternedString,
    fields: Vec<FFIField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FFIFunction {
    name: InternedString,
    params: Vec<FFIField>,
    ret_ty: FFIType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FFIField {
    ty: FFIType,
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
pub struct FFIType {
    value_type: FFIValueType,
    indirection: Indirection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FFIValueType {
    Void,
    Primitive(FFIPrimitive),
    Custom(InternedString),
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

