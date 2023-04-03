//! expands Quad into Quad<Quad>

use crate::{Block, DepthQuad, Node, Quad};
impl<'n> Quad<&'n Node> {
    pub(crate) fn children(&self) -> DepthQuad<&'n Quad<Block>, &'n Quad<Node>> {
        match self.map(Node::depth_quad) {
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
            } => DepthQuad::Inner(*depth, Quad { nw, ne, sw, se }),
            _ => panic!("inconsistent node depth"),
        }
    }
}
