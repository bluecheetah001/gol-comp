use std::fmt::Debug;

use bit_iter::BitIter;

use crate::{Pos, Rect};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
/// An 8x8 block of cells
///
/// # implementation details
/// stored in row-major format (see `Block::from_rows`)
pub struct Block {
    bits: u64,
}
impl Block {
    pub const WIDTH_LOG2: u8 = 3;
    pub const WIDTH: u64 = 1 << Self::WIDTH_LOG2;
    #[allow(clippy::cast_possible_wrap)] // value much smaller than i64::MAX
    pub const HALF_WIDTH: i64 = (Self::WIDTH / 2) as i64;
    pub const TRIVIAL_BOUNDING_RECT: Rect = Rect::symmetric_min_max(-4, 3);

    pub fn empty() -> Self {
        Self { bits: 0 }
    }
    /// constructs a block from a row-major sequence of bits
    ///
    /// The north-western most cell is the most significant bit.
    /// The south-eastern most cell is the least significant bit.
    /// A cell is alive if the corresponding bit is set.
    /// ```text
    /// 63 62 .. 56
    /// 55 54 .. 48
    /// :  :      :  
    /// 07 06 .. 00
    /// ```
    pub fn from_rows(bits: u64) -> Self {
        Self { bits }
    }
    pub fn to_rows(self) -> u64 {
        self.bits
    }
    /// constructs a block from a row-major sequence of bits
    ///
    /// The north-western most cell is the most significant bit in row 0.
    /// The south-eastern most cell is the least significant bit in row 7.
    /// A cell is alive if the corresponding bit is set.
    /// ```text
    /// 0,7 0,6 .. 0,0
    /// 1,7 1,6 .. 1,0
    ///  :   :      :  
    /// 7,7 7,6 .. 7,0
    /// ```
    pub fn from_rows_array(rows: [u8; 8]) -> Self {
        Self::from_rows(u64::from_be_bytes(rows))
    }
    pub fn to_rows_array(self) -> [u8; 8] {
        self.to_rows().to_be_bytes()
    }

    #[allow(clippy::cast_possible_wrap, reason = "values known to be < 64")]
    pub fn iter_alive(self) -> impl Iterator<Item = Pos> {
        BitIter::from(self.to_rows()).map(|bit| {
            let bit_col = (bit % 8) as i64;
            let bit_row = (bit / 8) as i64;
            Pos::new(3 - bit_col, -4 + bit_row)
        })
    }
}
impl Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [r0, r1, r2, r3, r4, r5, r6, r7] = self.to_rows_array();
        write!(
            f,
            "Block(0x{r0:02x}_{r1:02x}_{r2:02x}_{r3:02x}_{r4:02x}_{r5:02x}_{r6:02x}_{r7:02x})"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_empty() {
        assert_eq!(Block::from_rows(0), Block::empty());
        assert_eq!(Block::from_rows_array([0; 8]), Block::empty());
    }

    #[test]
    fn construct_same() {
        assert_eq!(
            Block::from_rows(0xff_dd_bb_99_77_55_33_11),
            Block::from_rows_array([0xff, 0xdd, 0xbb, 0x99, 0x77, 0x55, 0x33, 0x11])
        );
    }

    #[test]
    fn round_trip() {
        let val = 0xff_dd_bb_99_77_55_33_11;
        assert_eq!(val, Block::from_rows(val).to_rows());

        let val = [0xff, 0xdd, 0xbb, 0x99, 0x77, 0x55, 0x33, 0x11];
        assert_eq!(val, Block::from_rows_array(val).to_rows_array());
    }
}
