//! gets the center of a quad

use crate::{Block, DepthQuad, Node, Quad};

impl Node {
    // TODO center_at_depth is more efficnet if you can determine the goal depth ahead of time
    pub(crate) fn expand(&self) -> Node {
        self.expand_quad().into()
    }
    pub(crate) fn expand_quad(&self) -> Quad<Node> {
        match self.depth_quad() {
            DepthQuad::Leaf(leaf) => leaf.expand(),
            DepthQuad::Inner(_, inner) => inner.clone().expand(),
        }
    }
    pub(crate) fn center_at_depth(&self, depth: u8) -> Node {
        fn get_smaller(inner: Quad<&Node>, depth: u8) -> Node {
            match inner.children() {
                DepthQuad::Leaf(leaf) => {
                    assert_eq!(depth, 0);
                    leaf.center().copied().into()
                }
                DepthQuad::Inner(at_depth, inner) => {
                    if at_depth.get() == depth {
                        inner.center().cloned().into()
                    } else {
                        get_smaller(inner.center(), depth)
                    }
                }
            }
        }
        fn get_larger(inner: Quad<Node>, depth: u8) -> Node {
            let at_depth = inner.nw.depth();
            if at_depth + 1 == depth {
                inner.into()
            } else {
                get_larger(inner.expand(), depth)
            }
        }
        match self.depth().cmp(&depth) {
            // TODO handle self at depth 0
            std::cmp::Ordering::Less => match self.depth_quad() {
                DepthQuad::Leaf(leaf) => get_larger(leaf.expand(), depth),
                DepthQuad::Inner(_, inner) => get_larger(inner.clone(), depth),
            },
            std::cmp::Ordering::Equal => self.clone(),
            std::cmp::Ordering::Greater => get_smaller(self.inner().unwrap().as_ref(), depth),
        }
    }
}

impl Quad<&Node> {
    // a bit inconsistent with other methods, but all the places that use this want to own the result
    pub(crate) fn center(&self) -> DepthQuad<Block, Node> {
        match self.children() {
            DepthQuad::Leaf(leaf) => DepthQuad::Leaf(leaf.center().copied()),
            DepthQuad::Inner(depth, inner) => DepthQuad::Inner(depth, inner.center().cloned()),
        }
    }
}
impl Quad<Node> {
    pub(crate) fn expand(self) -> Quad<Node> {
        let empty = Node::empty(self.nw.depth());
        Quad {
            nw: Node::new(empty.clone(), empty.clone(), empty.clone(), self.nw),
            ne: Node::new(empty.clone(), empty.clone(), self.ne, empty.clone()),
            sw: Node::new(empty.clone(), self.sw, empty.clone(), empty.clone()),
            se: Node::new(self.se, empty.clone(), empty.clone(), empty),
        }
    }
}
impl Quad<Block> {
    pub(crate) fn expand(self) -> Quad<Node> {
        let empty = Block::empty();
        Quad {
            nw: Node::new(empty, empty, empty, self.nw),
            ne: Node::new(empty, empty, self.ne, empty),
            sw: Node::new(empty, self.sw, empty, empty),
            se: Node::new(self.se, empty, empty, empty),
        }
    }
}

impl Quad<Block> {
    #[allow(unused)]
    pub(crate) fn center(&self) -> Block {
        let nw = (self.nw.to_rows() & 0x00_00_00_00_0f_0f_0f_0f) << (4 * 8 + 4);
        let ne = (self.ne.to_rows() & 0x00_00_00_00_f0_f0_f0_f0) << (4 * 8 - 4);
        let sw = (self.sw.to_rows() & 0x0f_0f_0f_0f_00_00_00_00) >> (4 * 8 - 4);
        let se = (self.se.to_rows() & 0xf0_f0_f0_f0_00_00_00_00) >> (4 * 8 + 4);
        Block::from_rows(nw | ne | sw | se)
    }
}
impl Block {
    // is only used for tests
    pub(crate) fn expand(self) -> Quad<Block> {
        let bits = self.to_rows();
        let nw = Block::from_rows((bits & 0xf0_f0_f0_f0_00_00_00_00) >> (4 * 8 + 4));
        let ne = Block::from_rows((bits & 0x0f_0f_0f_0f_00_00_00_00) >> (4 * 8 - 4));
        let sw = Block::from_rows((bits & 0x00_00_00_00_f0_f0_f0_f0) << (4 * 8 - 4));
        let se = Block::from_rows((bits & 0x00_00_00_00_0f_0f_0f_0f) << (4 * 8 + 4));
        Quad { nw, ne, sw, se }
    }
}

impl<'t, T> Quad<&'t Quad<T>> {
    pub(crate) fn center(&self) -> Quad<&'t T> {
        Quad {
            nw: &self.nw.se,
            ne: &self.ne.sw,
            sw: &self.sw.ne,
            se: &self.se.nw,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Block, Node};

    #[test]
    fn leaf_center_at() {
        let b1 = Block::from_rows(0x01_02_04_08_10_20_40_80);
        let b2 = Block::from_rows(0x80_40_20_10_08_04_02_01);
        let b3 = Block::from_rows(0x01_01_01_01_01_01_01_01);
        let b4 = Block::from_rows(0x80_80_80_80_80_80_80_80);
        let leaf = Node::new(b1, b2, b3, b4);
        assert_eq!(leaf, leaf.center_at_depth(0));
        assert_eq!(leaf.expand(), leaf.center_at_depth(1));
        assert_eq!(leaf.expand().expand(), leaf.center_at_depth(2));
    }

    #[test]
    fn node0_center_at() {
        let b1 = Block::from_rows(0x01_02_04_08_10_20_40_80);
        let b2 = Block::from_rows(0x80_40_20_10_08_04_02_01);
        let b3 = Block::from_rows(0x01_01_01_01_01_01_01_01);
        let b4 = Block::from_rows(0x80_80_80_80_80_80_80_80);
        let leaf = Node::new(b1, b2, b3, b4);
        let node = Node::new(leaf.clone(), leaf.clone(), leaf.clone(), leaf);
        let center = Node::new(b4, b3, b2, b1);
        assert_eq!(center, node.center_at_depth(0));
        assert_eq!(node, node.center_at_depth(1));
        assert_eq!(node.expand(), node.center_at_depth(2));
    }
}
