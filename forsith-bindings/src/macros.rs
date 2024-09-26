#[macro_export]
macro_rules! define_vk_bitmasks {
    ($($bitmask:ident($bitflag_enum:ident) {$($bitflag:ident = $bit:expr $(,)?)+})+) => {
        $(
            #[repr(C)]
            pub struct $bitmask(pub crate::VkBitmask);

            impl crate::Bitmask for $bitmask {
                type Bitflag = $bitflag_enum;

                fn contains(&self, bitflag: $bitflag_enum) -> bool {
                    return self.0 & (bitflag as crate::VkBitflag) != 0
                }
            }
        )+

        crate::define_vk_enums!($(
            $bitflag_enum {$($bitflag = $bit,)+}
        )+);
    };
}

#[macro_export]
macro_rules! define_vk_enums {
    ($($enum:ident {$($variant:ident = $value:expr $(,)? )+})+) => {
        paste::item! {$(
            #[repr(u32)]
            pub enum $enum {
                $([<$variant:camel>] = $value,)+
            }
        )+}
    };
}

#[macro_export]
macro_rules! define_vk_structs {
    ($($visibility:vis $struct:ident {$($field:ident: $type:ty $(,)? )*})+) => {
        paste::item! {$(
            #[repr(C)]
            $visibility struct $struct {
                $( $visibility [<$field:snake>]: $type, )*
            }
        )+}
    };

    ($($visibility:vis $struct:ident($structure_type:expr) {$($field:ident: $type:ty $(,)? )*})+) => {
        crate::define_vk_structs!($(
            $visibility $struct {
            s_type: crate::structure_type::VkStructureType,
            p_next: *const std::ffi::c_void,
            $($field: $type,)*
        })+);

        $(
            #[allow(dead_code)]
            impl $struct {
                $visibility fn structure_type() -> crate::structure_type::VkStructureType {$structure_type}
            }
        )+
    };
}

#[macro_export]
macro_rules! define_extern_functions {
    ([$lib:expr]($extern:expr) $($vis:vis $function:ident($($arg:ident:  $type:ty $(,)?)*) $(-> $return_type:ty)? )+) => {
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
