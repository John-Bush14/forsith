mod slice;
pub use slice::BitSlice;

#[cfg(feature = "alloc")]
mod vec;
#[cfg(feature = "alloc")]
pub use vec::BitVec;

mod array;
pub use array::BitArray;
