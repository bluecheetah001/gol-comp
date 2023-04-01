use std::ops::BitOr;

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
                    Node::new_leaf(lhs.zip_map(*rhs, |lhs, rhs| lhs | rhs))
                }
                (crate::DepthQuad::Inner(depth, lhs), crate::DepthQuad::Inner(_, rhs)) => {
                    Node::new_depth_inner(
                        *depth,
                        lhs.as_ref()
                            .zip_map(rhs.as_ref(), |lhs, rhs| lhs.bitor_impl(rhs)),
                    )
                }
                _ => panic!("inconsistent depth"),
            }
        }
    }

    //     pub fn and(&self, rhs: &Node) -> Node {
    //         if self.depth() < rhs.depth() {
    //             bitand_impl(self, &rhs.center_at_depth(self.depth()))
    //         } else {
    //             bitand_impl(&self.center_at_depth(rhs.depth()), rhs)
    //         }
    //     }
    //     fn and_impl(&self, rhs: &Node) -> Node {
    //         if self.is_empty() {
    //             self.clone()
    //         } else if rhs.is_empty() {
    //             rhs.clone()
    //         } else {
    //             match (self.depth_quad(), rhs.depth_quad()) {
    //                 (crate::DepthQuad::Leaf(lhs), crate::DepthQuad::Leaf(rhs)) => {
    //                     Node::new_leaf(lhs.zip_map(*rhs, |lsh, rhs| lsh & rhs))
    //                 }
    //                 (crate::DepthQuad::Inner(depth, lhs), crate::DepthQuad::Inner(_, rhs)) => {
    //                     Node::new_depth_inner(*depth, lhs.as_ref().zip_map(rhs.as_ref(), bitand_impl))
    //                 }
    //                 _ => panic!("inconsistent depth"),
    //             }
    //         }
    //     }

    //     pub fn and_not(&self, rhs:&Node)->Node{
    //         self.and_not_impl(rhs.center_at_depth(self.depth()))
    //     }
    //     fn and_impl(&self, rhs: &Node) -> Node {
    //         if self.is_empty() || rhs.is_empty(){
    //             self.clone()
    //         } else {
    //             match (self.depth_quad(), rhs.depth_quad()) {
    //                 (crate::DepthQuad::Leaf(lhs), crate::DepthQuad::Leaf(rhs)) => {
    //                     todo!()
    //                 }
    //                 (crate::DepthQuad::Inner(depth, lhs), crate::DepthQuad::Inner(_, rhs)) => {
    //                     todo!()
    //                 }
    //                 _ => panic!("inconsistent depth"),
    //             }
    //         }
    //     }
}

impl BitOr for Block {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self::from_rows(self.to_rows() | rhs.to_rows())
    }
}
