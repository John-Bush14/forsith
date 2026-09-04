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

pub struct FFIItems<'a>(Vec<FFIItem<'a>>);

pub enum FFIItem<'a> {
    Struct(FFIStruct<'a>)
}

pub struct FFIStruct<'a> {
    name: &'a str,
    fields: Vec<FFIField<'a>>,
}

pub struct FFIFunction<'a> {
    name: &'a str,
    params: Vec<FFIField<'a>>,
    ret_ty: FFIType<'a>,
}

pub struct FFIField<'a> {
    ty: FFIType<'a>,
    meta: FFIFieldMetadata,
    optional: bool,
}

pub enum FFIFieldMetadata {
    None,
    Len(usize),
    LenField(&'static str),
}

pub struct FFIType<'a> {
    value_type: FFIValueType<'a>,
    indirection: Indirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub enum Indirection {
    None,
    Ptr,
    MutPtr,
    MutPtrMutPtr,
    MutPtrConstPtr,
    PtrPtr
}

