/// defines repr(C) bitflag enum(s) and their corresponding
/// repr(C) bitmask(s) with the [`crate::Bitmask`] trait implemented.
/// also makes all bitflags CamelCase
///
/// ## example use
/// ```ignore
/// define_vk_bitmasks!(
///     ExampleBitmask(ExampleBitmaskBitflag) {
///         EXAMPLE_BIT_FLAG = 0x0020030
///     }
/// );
/// ```
///
/// ## example outcome
/// ```ignore
/// #[repr(C)]
/// pub struct ExampleBitmask(pub VkBitmask);
///
/// impl Bitmask for ExampleBitmask {
///     type Bitflag = ExampleBitmaskBitFlag;
///
///     fn contains(&self, bitflag: ExampleBitmaskBitflag) {
///         return self.0 & (bitflag as crate::VkBitflag) != 0
///     }
/// }
///
/// define_vk_enums!(
///     ExampleBitmaskBitflag {
///         ExampleBitFlag = 0X0020030
///     }
/// );
/// ```
#[macro_export]
macro_rules! define_vk_bitmasks {
    ($($vis:vis $bitmask:ident($bitflag_enum:ident) {$($bitflag:ident = $bit:expr $(,)?)+})+) => {
        $(
            #[repr(C)]
            $vis struct $bitmask(pub crate::VkBitmask);

            impl crate::Bitmask for $bitmask {
                type Bitflag = $bitflag_enum;

                fn contains(&self, bitflag: $bitflag_enum) -> bool {
                    return self.0 & (bitflag as crate::VkBitflag) != 0
                }
            }
        )+

        crate::define_vk_enums!($(
            $vis $bitflag_enum {$($bitflag = $bit,)+}
        )+);
    };
}

/// defines repr(u32) enum(s) with all variants made CamelCase
///
/// ## example use
/// ```ignore
/// define_vk_enums!(
///     pub ExampleVkEnum {
///         ExampleVkVariant = 69420
///     }
/// );
/// ```
/// ## example outcome
/// ```ignore
/// #[repr(u32)]
/// pub enum ExampleVkEnum {
///     ExampleVkVariant = 69420
/// }
/// ```
#[macro_export]
macro_rules! define_vk_enums {
    ($($vis:vis $enum:ident {$($variant:ident = $value:expr $(,)? )+})+) => {
        paste::item! {$(
            #[repr(u32)]
            $vis enum $enum {
                $([<$variant:camel>] = $value,)+
            }
        )+}
    };
}

/// defines repr(C) struct with snake case fields.
/// optionally adds p_next + s_type fields and implements structure_type() func
/// by adding (VkStructureType::....) after struct name
///
/// ## example
/// ```ignore
/// define_vk_structs!(
///     pub ExampleVkStruct(VkStructureType::StructureTypeExampleVkStruct) { // (..) is optional
///         example_field: u8
///     }
/// );
/// ```
///
/// ## example outcome
/// ```ignore
/// #[repr(C)]
/// pub struct ExampleVkStruct {
///     pub s_type: VkStructureType::StructureTypeExampleVkStruct, // only if (..) is provided
///     pub p_next: *const c_void, // same thing
///     pub example_field: u8
/// }
///
/// #[allow(dead_code)]
/// impl ExampleVkStruct {
///     pub fn structure_type() -> VkStructureType {
///         VkStructureType::StructureTypeExampleVkStruct
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_vk_structs {
    ($($visibility:vis $struct:ident$(($structure_type:expr))? {$($field:ident: $type:ty $(,)? )*})+) => {
        paste::item! {$(
            #[repr(C)]
            $visibility struct $struct {
                $(
                    $visibility s_type: crate::structure_type::VkStructureType,
                    #[doc = concat!("This should be ", stringify!($structure_type), ".")]
                    $visibility p_next: *const std::ffi::c_void,
                )?
                $( $visibility [<$field:snake>]: $type, )*
            }
        )+}

        $($(
            #[allow(dead_code)]
            impl $struct {
                $visibility fn structure_type() -> crate::structure_type::VkStructureType {$structure_type}
            }
        )?)+
    };
}

/// defines private extern $extern function(s) linked to $lib
/// and then defines safe wrapper function(s) for it with a snake_case name
///
/// ## example use
/// ```ignore
/// define_extern_functions!(["vulkan"]("C")
///     pub TestExternFunction(
///         test_field: u8
///     ) -> u16;
/// );
/// ```
///
/// ## example outcome
/// ```ignore
/// #[link(name = "vulkan")]
/// extern "C" {
///     fn TestExternFunction(
///         test_field: u8
///     ) -> u16;
/// }
///
/// pub fn test_extern_function(test_field: u8) -> u16 {
///     unsafe {TestExternFunction(test_field)}
/// }
/// ```
#[macro_export]
macro_rules! define_extern_functions {
    ([$lib:expr]($extern:expr) $($vis:vis $function:ident($($arg:ident:  $type:ty $(,)?)*) $(-> $return_type:ty)? $(;)? )+) => {
        #[link(name = $lib)]
        extern $extern {$(
            #[allow(clashing_extern_declarations)]
            fn $function($($arg: $type,)*) $(-> $return_type)?;
        )+}

        paste::item! {$(
            $vis fn [<$function:snake>]($($arg: $type,)*) $(-> $return_type)? {
                unsafe {$function($($arg,)*)}
            }
        )+}
    };
}


#[cfg(test)]
mod macro_tests {
    define_vk_enums!(TestVkEnum {
        ONE = 1,
        TWO = 2
    });

    #[test]
    fn use_defined_vk_enum() {
        assert!(TestVkEnum::One as u32 == 1);
        assert!(TestVkEnum::Two as u32 == 2);
    }


    use crate::{instance::VkInstance, structure_type::VkStructureType, vk_result::VkResult, Bitmask};

    define_vk_bitmasks!(
        TestBitmask(TestBitflag) {
            DEAD = 1,
            ALIVE = 2
        }
    );

    #[test]
    fn use_defined_vk_bitmask() {
        let alive = TestBitmask(2);

        assert!(alive.contains(TestBitflag::Alive));
        assert!(!alive.contains(TestBitflag::Dead));
    }


    define_vk_structs!(
        pub TestVkStruct {
            field: *const u32
        }
    );

    define_vk_structs!(
        pub TestVkCreateStruct(VkStructureType::VkStructureTypeApplicationInfo) {
            field: *const u32
        }
    );

    #[test]
    fn use_defined_vk_struct() {
        let test_vk_create_struct = TestVkCreateStruct {
            s_type: TestVkCreateStruct::structure_type(),
            p_next: std::ptr::null(),
            field: std::ptr::null(),
        };

        assert!(test_vk_create_struct.s_type as u32 == VkStructureType::VkStructureTypeApplicationInfo as u32);

        let _test_vk_struct = TestVkStruct {field: std::ptr::null()};
    }


    use std::ffi::c_void;

    define_extern_functions!(["vulkan"]("C") vkCreateInstance(_a: *const c_void, _b: *const c_void, instance: *mut VkInstance) -> VkResult);

    #[test]
    fn use_defined_c_function() {
        let result = vk_create_instance(std::ptr::null(), std::ptr::null(), std::ptr::null_mut());
        assert_eq!(result as u32, VkResult::VkErrorInitializationFailed as u32);
    }
}
