#[macro_export]
macro_rules! define_vk_bitmask {
    ($bitmask:ident($bitflag_enum:ident) {$($bitflag:ident = $bit:expr $(,)?)+}) => {
        pub struct $bitmask(crate::VkBitmask);

        impl crate::Bitmask for $bitmask {
            type Bitflag = $bitflag_enum;

            fn contains(&self, bitflag: $bitflag_enum) -> bool {
                return self.0 & (bitflag as crate::VkBitflag) != 0
            }
        }

        crate::define_vk_enum!($bitflag_enum {$($bitflag = $bit,)+});
    };
}

#[macro_export]
macro_rules! define_vk_enum {
    ($enum:ident {$($variant:ident = $value:expr $(,)? )+}) => {
        paste::item! {
            #[repr(u32)]
            pub enum $enum {
                $([<$variant:camel>] = $value,)+
            }
        }
    };
}

#[macro_export]
macro_rules! define_vk_struct {
    ($visibility:ident $struct:ident {$($field:ident: $type:ty $(,)? )*}) => {
        paste::item! {
            #[repr(C)]
            $visibility struct $struct {
                $($visibility [<$field:snake>]: $type,)*
            }
        }
    };

    ($visibility:ident $struct:ident($structure_type:ident) {$($field:ident: $type:ty $(,)? )*}) => {
        #[allow(dead_code)]
        use crate::structure_type::VkStructureType;

        crate::define_vk_struct!($visibility $struct {
            s_type: VkStructureType,
            p_next: *const std::ffi::c_void,
            $($field: $type,)*
        });

        paste::item! {impl $struct {
            $visibility fn structure_type() -> VkStructureType {VkStructureType::[<$structure_type:camel>]}
        }}
    };
}


#[cfg(test)]
mod macro_tests {
    define_vk_enum!(TestVkEnum {
        ONE = 1,
        TWO = 2
    });

    #[test]
    fn use_defined_vk_enum() {
        assert!(TestVkEnum::One as u32 == 1);
        assert!(TestVkEnum::Two as u32 == 2);
    }


    use crate::Bitmask;

    define_vk_bitmask!(
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


    define_vk_struct!(
        pub TestVkStruct {
            field: *const u32
        }
    );

    define_vk_struct!(
        pub TestVkCreateStruct(VK_STRUCTURE_TYPE_APPLICATION_INFO) {
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
}
