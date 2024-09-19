#[macro_export]
macro_rules! pub_consts {
    ($type:ident, $($name:ident = $value:expr,)*) => {
        $(
            pub const $name: $type = $value;
        )*
    };
}

#[macro_export]
macro_rules! vk_create_info {
    ($pub:ident $name:ident {$(
        $field:ident: $r#type:ty $(,)?
    )*}) => {
        #[repr(C)]
        $pub struct $name {
            pub s_type: crate::structure_type::VkStructureType,
            pub p_next: *const std::ffi::c_void,
            $(
            pub $field: $r#type,
            )*
        }
    };
}
