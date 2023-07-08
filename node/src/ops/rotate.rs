use crate::{Block, DepthQuad, Node, Quad};

// TODO memoize or special case empty?

// TODO many (all after setup?) Node ops just delegate, could use a macro to generalize

impl Node {
    pub fn flip_h(&self) -> Node {
        match self.depth_quad() {
            DepthQuad::Leaf(leaf) => Node::new_leaf(leaf.flip_h()),
            DepthQuad::Inner(depth, inner) => Node::new_depth_inner(*depth, inner.flip_h()),
        }
    }
    pub fn flip_v(&self) -> Node {
        match self.depth_quad() {
            DepthQuad::Leaf(leaf) => Node::new_leaf(leaf.flip_v()),
            DepthQuad::Inner(depth, inner) => Node::new_depth_inner(*depth, inner.flip_v()),
        }
    }
    pub fn rotate_cw(&self) -> Node {
        match self.depth_quad() {
            DepthQuad::Leaf(leaf) => Node::new_leaf(leaf.rotate_cw()),
            DepthQuad::Inner(depth, inner) => Node::new_depth_inner(*depth, inner.rotate_cw()),
        }
    }
    pub fn rotate_180(&self) -> Node {
        match self.depth_quad() {
            DepthQuad::Leaf(leaf) => Node::new_leaf(leaf.rotate_180()),
            DepthQuad::Inner(depth, inner) => Node::new_depth_inner(*depth, inner.rotate_180()),
        }
    }
    pub fn rotate_ccw(&self) -> Node {
        match self.depth_quad() {
            DepthQuad::Leaf(leaf) => Node::new_leaf(leaf.rotate_ccw()),
            DepthQuad::Inner(depth, inner) => Node::new_depth_inner(*depth, inner.rotate_ccw()),
        }
    }
}

// TODO most (all?) ops have duplicated code between `Quad<Node>` and `Quad<Block>`, could use a trait to generalize
// but I don't really want to have to bring in a bunch of different traits, I'd like the crate to just export
// Node, direct methods, and raw data (which only have constructors and destructors)
// although some methods (like these) do directly apply to Quad and Block, many have some sort of initialization that only makes sense for Node
// Pos and Rect will have direct methods, but they are mostly separate from the Node, DepthQuad, Quad, Block heirarchy

impl Quad<Node> {
    fn flip_h(&self) -> Quad<Node> {
        Quad {
            nw: self.ne.flip_h(),
            ne: self.nw.flip_h(),
            sw: self.se.flip_h(),
            se: self.sw.flip_h(),
        }
    }
    fn flip_v(&self) -> Quad<Node> {
        Quad {
            nw: self.sw.flip_v(),
            ne: self.se.flip_v(),
            sw: self.nw.flip_v(),
            se: self.ne.flip_v(),
        }
    }
    fn rotate_cw(&self) -> Quad<Node> {
        Quad {
            nw: self.sw.rotate_cw(),
            ne: self.nw.rotate_cw(),
            sw: self.se.rotate_cw(),
            se: self.ne.rotate_cw(),
        }
    }
    fn rotate_180(&self) -> Quad<Node> {
        Quad {
            nw: self.se.rotate_180(),
            ne: self.sw.rotate_180(),
            sw: self.ne.rotate_180(),
            se: self.nw.rotate_180(),
        }
    }
    fn rotate_ccw(&self) -> Quad<Node> {
        Quad {
            nw: self.ne.rotate_ccw(),
            ne: self.se.rotate_ccw(),
            sw: self.nw.rotate_ccw(),
            se: self.sw.rotate_ccw(),
        }
    }
}

impl Quad<Block> {
    fn flip_h(&self) -> Quad<Block> {
        Quad {
            nw: self.ne.flip_h(),
            ne: self.nw.flip_h(),
            sw: self.se.flip_h(),
            se: self.sw.flip_h(),
        }
    }
    fn flip_v(&self) -> Quad<Block> {
        Quad {
            nw: self.sw.flip_v(),
            ne: self.se.flip_v(),
            sw: self.nw.flip_v(),
            se: self.ne.flip_v(),
        }
    }
    fn rotate_cw(&self) -> Quad<Block> {
        Quad {
            nw: self.sw.rotate_cw(),
            ne: self.nw.rotate_cw(),
            sw: self.se.rotate_cw(),
            se: self.ne.rotate_cw(),
        }
    }
    fn rotate_180(&self) -> Quad<Block> {
        Quad {
            nw: self.se.rotate_180(),
            ne: self.sw.rotate_180(),
            sw: self.ne.rotate_180(),
            se: self.nw.rotate_180(),
        }
    }
    fn rotate_ccw(&self) -> Quad<Block> {
        Quad {
            nw: self.ne.rotate_ccw(),
            ne: self.se.rotate_ccw(),
            sw: self.nw.rotate_ccw(),
            se: self.sw.rotate_ccw(),
        }
    }
}

impl Block {
    fn flip_h(self) -> Block {
        let rows = self.to_rows();
        let rows =
            (rows & 0xf0_f0_f0_f0_f0_f0_f0_f0) >> 4 | (rows & 0x0f_0f_0f_0f_0f_0f_0f_0f) << 4;
        let rows =
            (rows & 0xcc_cc_cc_cc_cc_cc_cc_cc) >> 2 | (rows & 0x33_33_33_33_33_33_33_33) << 2;
        let rows =
            (rows & 0xaa_aa_aa_aa_aa_aa_aa_aa) >> 1 | (rows & 0x55_55_55_55_55_55_55_55) << 1;
        Self::from_rows(rows)
    }
    fn flip_v(self) -> Block {
        let rows = self.to_rows();
        let rows =
            (rows & 0xff_ff_ff_ff_00_00_00_00) >> 32 | (rows & 0x00_00_00_00_ff_ff_ff_ff) << 32;
        let rows =
            (rows & 0xff_ff_00_00_ff_ff_00_00) >> 16 | (rows & 0x00_00_ff_ff_00_00_ff_ff) << 16;
        let rows =
            (rows & 0xff_00_ff_00_ff_00_ff_00) >> 8 | (rows & 0x00_ff_00_ff_00_ff_00_ff) << 8;
        Self::from_rows(rows)
    }
    // TODO im pretty sure I know how to implement these directly now
    // rows = (rows & nw >> 4) | (rows & ne >> 32) | (rows & sw << 32) | (rows & se << 4)
    // and then 'recursing' into 2x2 and 1x1 blocks
    fn rotate_cw(self) -> Block {
        self.flip_d().flip_h()
    }
    fn rotate_180(self) -> Block {
        self.flip_v().flip_h()
    }
    fn rotate_ccw(self) -> Block {
        self.flip_d().flip_v()
    }
    fn flip_d(self) -> Block {
        let rows = self.to_rows();
        let rows = (rows & 0x0f_0f_0f_0f_00_00_00_00) >> 28
            | (rows & 0x00_00_00_00_f0_f0_f0_f0) << 28
            | (rows & 0xf0_f0_f0_f0_0f_0f_0f_0f);
        let rows = (rows & 0x33_33_00_00_33_33_00_00) >> 14
            | (rows & 0x00_00_cc_cc_00_00_cc_cc) << 14
            | (rows & 0xcc_cc_33_33_cc_cc_33_33);
        let rows = (rows & 0x55_00_55_00_55_00_55_00) >> 7
            | (rows & 0x00_aa_00_aa_00_aa_00_aa) << 7
            | (rows & 0xaa_55_aa_55_aa_55_aa_55);
        Self::from_rows(rows)
    }
}

#[cfg(test)]
mod test {
    use crate::Block;

    #[test]
    fn all_blocks() {
        let blocks = [
            Block::from_rows(0x80_c0_e0_f0_00_00_00_00),
            Block::from_rows(0xf0_70_30_10_00_00_00_00),
            Block::from_rows(0x0f_0e_0c_08_00_00_00_00),
            Block::from_rows(0x01_03_07_0f_00_00_00_00),
            Block::from_rows(0x00_00_00_00_0f_07_03_01),
            Block::from_rows(0x00_00_00_00_08_0c_0e_0f),
            Block::from_rows(0x00_00_00_00_10_30_70_f0),
            Block::from_rows(0x00_00_00_00_f0_e0_c0_80),
        ];
        for i in 0..8 {
            assert_eq!(blocks[i].rotate_cw(), blocks[(i + 2) % 8], "i={i} cw");
            assert_eq!(blocks[i].rotate_180(), blocks[(i + 4) % 8], "i={i} 180");
            assert_eq!(blocks[i].rotate_ccw(), blocks[(i + 6) % 8], "i={i} ccw");
            assert_eq!(blocks[i].flip_v(), blocks[7 - i], "i={i} v");
            assert_eq!(blocks[i].flip_h(), blocks[(11 - i) % 8], "i={i} h");
        }
    }
}
