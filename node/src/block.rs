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
    pub const WIDTH: u64 = 1 << Block::WIDTH_LOG2;

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
// impl Quad<Block> {
//     /// convert a quad of blocks into a single block by unioning 2x2 cells into a single cell
//     pub fn zoom_out(&self) -> Block {
//         fn zoom_out_h(w: Block, e: Block) -> Block {
//             let w = w.0 | (w.0 << 1);
//             let e = e.0 | (e.0 << 1);
//             let c0 = (w << 0) & 0x80_80_80_80_80_80_80_80;
//             let c1 = (w << 1) & 0x40_40_40_40_40_40_40_40;
//             let c2 = (w << 2) & 0x20_20_20_20_20_20_20_20;
//             let c3 = (w << 3) & 0x10_10_10_10_10_10_10_10;
//             let c4 = (e >> 4) & 0x08_08_08_08_08_08_08_08;
//             let c5 = (e >> 3) & 0x04_04_04_04_04_04_04_04;
//             let c6 = (e >> 2) & 0x02_02_02_02_02_02_02_02;
//             let c7 = (e >> 1) & 0x01_01_01_01_01_01_01_01;
//             Block(c0 | c1 | c2 | c3 | c4 | c5 | c6 | c7)
//         }
//         fn zoom_out_v(n: Block, s: Block) -> Block {
//             let n = n.0 | (n.0 << 8);
//             let s = s.0 | (s.0 << 8);
//             let r0 = (n << 00) & 0xff_00_00_00_00_00_00_00;
//             let r1 = (n << 08) & 0x00_ff_00_00_00_00_00_00;
//             let r2 = (n << 16) & 0x00_00_ff_00_00_00_00_00;
//             let r3 = (n << 24) & 0x00_00_00_ff_00_00_00_00;
//             let r4 = (s >> 32) & 0x00_00_00_00_ff_00_00_00;
//             let r5 = (s >> 24) & 0x00_00_00_00_00_ff_00_00;
//             let r6 = (s >> 16) & 0x00_00_00_00_00_00_ff_00;
//             let r7 = (s >> 08) & 0x00_00_00_00_00_00_00_ff;
//             Block(r0 | r1 | r2 | r3 | r4 | r5 | r6 | r7)
//         }
//         let Self { nw, ne, sw, se } = *self;
//         zoom_out_v(zoom_out_h(nw, ne), zoom_out_h(sw, se))
//     }
// }
