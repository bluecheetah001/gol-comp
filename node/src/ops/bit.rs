use std::ops::{BitOr, BitXor};

use crate::{Block, Node, Population};

// could override & | ^, but fns are clearer since they allow `and_not` even though `not` isn't possible (don't support infinite field of alive)
impl Node {
    pub fn or(&self, rhs: &Node) -> Node {
        if self.depth() > rhs.depth() {
            self.bitor_impl(&rhs.center_at_depth(self.depth()))
        } else {
            self.center_at_depth(rhs.depth()).bitor_impl(rhs)
        }
    }
    fn bitor_impl(&self, rhs: &Node) -> Node {
        if self.is_empty() {
            rhs.clone()
        } else if rhs.is_empty() {
            self.clone()
        } else {
            match (self.depth_quad(), rhs.depth_quad()) {
                (crate::DepthQuad::Leaf(lhs), crate::DepthQuad::Leaf(rhs)) => {
                    lhs.zip_map(*rhs, Block::bitor).into()
                }
                (crate::DepthQuad::Inner(depth, lhs), crate::DepthQuad::Inner(_, rhs)) => {
                    Node::new_depth_inner(
                        *depth,
                        lhs.as_ref().zip_map(rhs.as_ref(), Node::bitor_impl),
                    )
                }
                _ => panic!("inconsistent depth"),
            }
        }
    }

    pub fn xor(&self, rhs: &Node) -> Node {
        if self.depth() > rhs.depth() {
            self.bitxor_impl(&rhs.center_at_depth(self.depth()))
        } else {
            self.center_at_depth(rhs.depth()).bitxor_impl(rhs)
        }
    }
    fn bitxor_impl(&self, rhs: &Node) -> Node {
        if self.is_empty() {
            rhs.clone()
        } else if rhs.is_empty() {
            self.clone()
        } else {
            match (self.depth_quad(), rhs.depth_quad()) {
                (crate::DepthQuad::Leaf(lhs), crate::DepthQuad::Leaf(rhs)) => {
                    lhs.zip_map(*rhs, Block::bitxor).into()
                }
                (crate::DepthQuad::Inner(depth, lhs), crate::DepthQuad::Inner(_, rhs)) => {
                    Node::new_depth_inner(
                        *depth,
                        lhs.as_ref().zip_map(rhs.as_ref(), Node::bitxor_impl),
                    )
                }
                _ => panic!("inconsistent depth"),
            }
        }
    }
}

impl BitOr for Block {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self::from_rows(self.to_rows() | rhs.to_rows())
    }
}

impl BitXor for Block {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::from_rows(self.to_rows() ^ rhs.to_rows())
    }
}

// TODO testing!
