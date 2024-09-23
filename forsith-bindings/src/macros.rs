#[macro_export]
macro_rules! define_vk_bitmask {
    ($bitmask:ident($bitflag_enum:ident): {$($bitflag:ident = $bit:expr $(,)?)*}) => {
        pub struct $bitmask(crate::VkBitmask);

        impl crate::Bitmask for $bitmask {
            type Bitflag = $bitflag_enum;

            fn contains(&self, bitflag: $bitflag_enum) -> bool {
                return self.0 & (bitflag as crate::VkBitflag) != 0
            }
        }

        #[repr(u32)]
        pub enum $bitflag_enum { $(
            $bitflag = $bit,
        )*}
    };
}

#[macro_export]
macro_rules! define_vk_enum {
    ($enum:ident {$($variant:ident = $value:expr $(,)?)*}) => {
        #[repr(u32)]
        pub enum $enum {
            $($variant = $value,)*
        }
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
        assert!(TestVkEnum::ONE as u32 == 1);
        assert!(TestVkEnum::TWO as u32 == 2);
    }


    use crate::Bitmask;

    define_vk_bitmask!(
        TestBitmask(TestBitflag): {
            DEAD = 1,
            ALIVE = 2
        }
    );

    #[test]
    fn use_defined_vk_bitmask() {
        let alive = TestBitmask(2);

        assert!(alive.contains(TestBitflag::ALIVE));
        assert!(!alive.contains(TestBitflag::DEAD));
    }
}
