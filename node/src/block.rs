use std::fmt::Debug;

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
}
impl Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [r0, r1, r2, r3, r4, r5, r6, r7] = self.to_rows_array();
        write!(
            f,
            "Block({r0:02x} {r1:02x} {r2:02x} {r3:02x} {r4:02x} {r5:02x} {r6:02x} {r7:02x})"
        )
    }
}
