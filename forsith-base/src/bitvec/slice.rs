use std::ops::Index;

pub struct BitSlice {
    data: [()],
}

impl Index<usize> for BitSlice {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        let (byte, bit_i) = self.get_byte_bit_index(index);

        if byte & (1 << bit_i) != 0 {
            &true
        } else {
            &false
        }
    }
}

impl BitSlice {
    #[must_use]
    pub const fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr().cast()
    }
    #[must_use]
    pub const fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr().cast()
    }

    const fn get_byte_bit_index(&self, index: usize) -> (u8, usize) {
        assert!(index < self.bits(), "index out of bounds");
        let index = index / 8;
        let byte = unsafe {*self.as_ptr().add(index)};

        (byte, index % 8)
    }

    const fn get_mut_byte_bit_index(&mut self, index: usize) -> (&mut u8, usize) {
        assert!(index < self.bits(), "index out of bounds");
        let index = index / 8;
        let byte = unsafe {&mut *self.as_mut_ptr().add(index)};

        (byte, index % 8)
    }

    #[must_use]
    pub const fn bits(&self) -> usize {
        self.data.len()
    }

    #[must_use]
    pub const fn len_bytes(&self) -> usize {
        self.data.len().div_ceil(8)
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[must_use]
    pub const fn from_raw_parts<'a>(ptr: *const u8, bits: usize) -> &'a Self {
        unsafe {
            let wptr = std::ptr::slice_from_raw_parts(ptr, bits);

            &*(wptr as *const Self)
        }
    }

    #[must_use]
    pub fn from_raw_parts_mut<'a>(ptr: *mut u8, bits: usize) -> &'a mut Self {
        unsafe {
            let wptr = std::ptr::slice_from_raw_parts_mut(ptr, bits);

            &mut *(wptr as *mut Self)
        }
    }

    pub const fn set(&mut self, index: usize, bit: bool) {
        let (byte, bit_i) = self.get_mut_byte_bit_index(index);

        if bit {
            *byte |= 1 << bit_i;
        } else {
             *byte &= !(1 << bit_i);
        }
    }
}
