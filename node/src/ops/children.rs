//! expands Quad into Quad<Quad>

use crate::{Block, DepthQuad, Node, Quad};

impl Quad<Node> {
    pub(crate) fn children(&self) -> DepthQuad<Quad<Block>, Quad<Node>> {
        match self.as_ref().map(|node| node.depth_quad().clone()) {
            Quad {
                nw: DepthQuad::Leaf(nw),
                ne: DepthQuad::Leaf(ne),
                sw: DepthQuad::Leaf(sw),
                se: DepthQuad::Leaf(se),
            } => DepthQuad::Leaf(Quad { nw, ne, sw, se }),
            Quad {
                nw: DepthQuad::Inner(depth, nw),
                ne: DepthQuad::Inner(_, ne),
                sw: DepthQuad::Inner(_, sw),
                se: DepthQuad::Inner(_, se),
            } => DepthQuad::Inner(depth, Quad { nw, ne, sw, se }),
            _ => panic!("inconsistent node depth"),
        }
    }
}
