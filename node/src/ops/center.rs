//! gets the center of a quad

use crate::{Block, DepthQuad, Node, Quad};

impl Node {
    // TODO center_at_depth is more efficnet if you can determine the goal depth ahead of time
    pub(crate) fn expand(&self) -> Node {
        match self.depth_quad() {
            DepthQuad::Leaf(leaf) => leaf.expand(Block::empty()).map(Node::new_leaf).into(),
            DepthQuad::Inner(depth, inner) => inner
                .clone()
                .expand(Node::empty(depth.get() - 1))
                .map(Node::new_inner)
                .into(),
        }
    }
    pub fn center_at_depth(&self, depth: u8) -> Node {
        fn get_smaller(inner: Quad<Node>, depth: u8) -> Node {
            match inner.children() {
                DepthQuad::Leaf(leaf) => {
                    assert_eq!(depth, 0);
                    leaf.center().into()
                }
                DepthQuad::Inner(at_depth, inner) => {
                    if at_depth.get() == depth {
                        inner.center().into()
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
                get_larger(
                    inner
                        .expand(Node::empty(at_depth))
                        .map(|quad| Node::new_inner(quad)),
                    depth,
                )
            }
        }
        match self.depth().cmp(&depth) {
            std::cmp::Ordering::Less => get_smaller(self.inner().unwrap().clone(), depth),
            std::cmp::Ordering::Equal => self.clone(),
            // TODO handle self at depth 0
            std::cmp::Ordering::Greater => get_larger(self.inner().unwrap().clone(), depth),
        }
    }
}

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
