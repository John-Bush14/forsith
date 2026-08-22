use std::simd::Simd;

use const_for::const_for;

#[allow(clippy::unreadable_literal)]
const AAN_SCALE: [f64; 8] = [
    1.0,
    1.3870398453221475,
    1.3065629648763766,
    1.1758756024193588,
    1.0,
    0.7856949583871022,
    0.541196100146197,
    0.2758993792829431
];
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
const fn scale(x: f64) -> i32 {(x * (1 << 12) as f64 + 0.5f64)as i32}

#[derive(Clone, Debug)]
pub struct IdctTable([Simd<i32, 8>; 8]);
impl Default for IdctTable {
    fn default() -> Self {Self::DEFAULT}
}

impl IdctTable {
    pub const DEFAULT: Self = {
        let mut table = [[0i32; 8]; 8];
        const_for!(u in 0..8 => {
            const_for!(v in 0..8 => {
                table[u][v] = scale(AAN_SCALE[u] * AAN_SCALE[v]);
            });
        });

        Self([
            Simd::from_array(table[0]),
            Simd::from_array(table[1]),
            Simd::from_array(table[2]),
            Simd::from_array(table[3]),
            Simd::from_array(table[4]),
            Simd::from_array(table[5]),
            Simd::from_array(table[6]),
            Simd::from_array(table[7]),
        ])
    };

    pub fn load(quant_table: [Simd<i32, 8>; 8]) -> Self {
        let mut table = Self::default();

        #[rustc_unroll]
        for (u, row) in table.0.iter_mut().enumerate() {
            *row *= quant_table[u];
        }

        table
    }
}
