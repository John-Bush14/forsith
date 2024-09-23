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


#[cfg(test)]
mod macro_tests {
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
