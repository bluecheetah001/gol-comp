// use std::num::NonZeroU8;

// use node::{Block, DepthQuad, Node, Population, Quad};

// fn reduce(node: &Node) -> Quad<Block> {
//     match node.depth_quad() {
//         DepthQuad::Leaf(leaf) => *leaf,
//         DepthQuad::Inner(_, inner) => inner.as_ref().map(reduce_1),
//     }
// }
// fn reduce_1(node: &Node) -> Block {
//     if node.is_empty() {
//         return Block::empty();
//     }
//     match node.depth_quad() {
//         DepthQuad::Leaf(leaf) => leaf.zoom_out(),
//         DepthQuad::Inner(_, inner) => {
//             let mut rows = 0_u64;
//         }
//     }
// }
