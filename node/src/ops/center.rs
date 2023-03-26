//! gets the center of a quad

use crate::{Block, DepthQuad, Node, Quad};

impl Quad<Node> {
    pub(crate) fn center(&self) -> DepthQuad<Block, Node> {
        match self.children() {
            DepthQuad::Leaf(leaf) => DepthQuad::Leaf(leaf.center()),
            DepthQuad::Inner(depth, inner) => DepthQuad::Inner(depth, inner.center()),
        }
    }
}

// impl Quad<Block> {
//     pub(crate) fn center(&self) -> Block {
//         let nw = (self.nw.to_rows() & 0x00_00_00_00_0f_0f_0f_0f) << (4 * 8 + 4);
//         let ne = (self.ne.to_rows() & 0x00_00_00_00_f0_f0_f0_f0) << (4 * 8 - 4);
//         let sw = (self.sw.to_rows() & 0x0f_0f_0f_0f_00_00_00_00) >> (4 * 8 - 4);
//         let se = (self.se.to_rows() & 0xf0_f0_f0_f0_00_00_00_00) >> (4 * 8 + 4);
//         Block::from_rows(nw | ne | sw | se)
//     }
// }

impl<T> Quad<Quad<T>>
where
    T: Clone,
{
    // TODO a couple of places may be more efficient if this returned Quad<&T>
    // but that will take a decent amount of work for not much gain
    pub(crate) fn center(&self) -> Quad<T> {
        Quad {
            nw: self.nw.se.clone(),
            ne: self.ne.sw.clone(),
            sw: self.sw.ne.clone(),
            se: self.se.nw.clone(),
        }
    }
}
