use std::cell::RefCell;
use std::hash::Hash;
use std::num::NonZeroU8;
use std::rc::{Rc, Weak};
use weak_table::WeakHashSet;

use crate::block::Block;
use crate::ops::Population;
use crate::quad::{DepthQuad, Quad};

thread_local! {
    static NODE_CACHE: RefCell<WeakHashSet<WeakNode>> = RefCell::new(WeakHashSet::new());
    static EMPTY_NODES: Vec<Node> = gen_empty_nodes();
}

type NodeData = (DepthQuad<Block, Node>, u64);
#[derive(Clone, Eq)]
pub struct Node(Rc<NodeData>);

impl Node {
    pub(crate) const MAX_WIDTH_LOG2: u8 = 63;
    /// the offset between depth and width_log2
    pub(crate) const MIN_WIDTH_LOG2: u8 = Block::WIDTH_LOG2 + 1;
    pub(crate) const MAX_DEPTH: u8 = Node::MAX_WIDTH_LOG2 - Node::MIN_WIDTH_LOG2;

    pub fn new(data: DepthQuad<Block, Node>) -> Self {
        assert!(data.depth() <= Node::MAX_DEPTH);
        NODE_CACHE.with_borrow_mut(|node_cache| {
            node_cache.get(&data).unwrap_or_else(|| {
                data.validate_depth();
                let population = data.population();
                let node = Self(Rc::new((data, population)));
                node_cache.insert(node.clone());
                node
            })
        })
    }
    pub fn new_leaf(data: Quad<Block>) -> Self {
        Self::new(DepthQuad::Leaf(data))
    }
    pub fn new_inner(data: Quad<Node>) -> Self {
        let depth = NonZeroU8::new(data.nw.depth() + 1).unwrap();
        Self::new_depth_inner(depth, data)
    }
    pub fn new_depth_inner(depth: NonZeroU8, data: Quad<Node>) -> Self {
        Self::new(DepthQuad::Inner(depth, data))
    }
    pub fn empty(depth: u8) -> Self {
        assert!(depth <= Node::MAX_DEPTH);
        EMPTY_NODES.with(|empty_nodes| empty_nodes[depth as usize].clone())
    }
    fn as_ref(&self) -> &NodeData {
        &self.0
    }
}
impl From<DepthQuad<Block, Node>> for Node {
    fn from(data: DepthQuad<Block, Node>) -> Self {
        Self::new(data)
    }
}
impl From<Quad<Block>> for Node {
    fn from(data: Quad<Block>) -> Self {
        Self::new_leaf(data)
    }
}
impl From<Quad<Node>> for Node {
    fn from(data: Quad<Node>) -> Self {
        Self::new_inner(data)
    }
}

fn gen_empty_nodes() -> Vec<Node> {
    let empty_leaf = Node::new_leaf(Quad {
        nw: Block::empty(),
        ne: Block::empty(),
        sw: Block::empty(),
        se: Block::empty(),
    });
    std::iter::successors(Some(empty_leaf), |empty| {
        if empty.depth() < Node::MAX_DEPTH {
            Some(Node::new_inner(Quad {
                nw: empty.clone(),
                ne: empty.clone(),
                sw: empty.clone(),
                se: empty.clone(),
            }))
        } else {
            None
        }
    })
    .collect()
}

impl Node {
    pub fn depth_quad(&self) -> &DepthQuad<Block, Node> {
        &self.as_ref().0
    }
    pub fn depth(&self) -> u8 {
        self.depth_quad().depth()
    }
    pub fn width(&self) -> u64 {
        1 << (self.width_log2())
    }
    pub fn half_width(&self) -> i64 {
        1 << (self.width_log2() - 1)
    }
    pub fn width_log2(&self) -> u8 {
        self.depth() + Node::MIN_WIDTH_LOG2
    }
    pub fn leaf(&self) -> Option<&Quad<Block>> {
        self.depth_quad().leaf()
    }
    pub fn inner(&self) -> Option<&Quad<Node>> {
        self.depth_quad().inner()
    }

    // pub fn or(&self, other: &Node, ctx: &mut NodeCache) -> Self {
    //     fn equal_leaf_or(a: &Quad<State>, b: &Quad<State>) -> Node {
    //         Node::new_leaf(a.zip_map(*b, |a, b| a | b))
    //     }
    //     fn equal_inner_or(a: &Quad<Node>, b: &Quad<Node>) -> Node {
    //         Node::new_inner(a.as_ref().zip_map(b.as_ref(), |a, b| equal_node_or(a, b)))
    //     }
    //     fn equal_node_or(a: &Node, b: &Node) -> Node {
    //         match (a.depth_quad(), b.depth_quad()) {
    //             (DepthQuad::Leaf(a), DepthQuad::Leaf(b)) => equal_leaf_or(a, b),
    //             (DepthQuad::Inner(_, a), DepthQuad::Inner(_, b)) => equal_inner_or(a, b),
    //             _ => panic!("node depths were not equal"),
    //         }
    //     }
    //     fn corner_leaf_or(a: &Quad<State>, b: &Quad<State>, corner: Quadrant) -> Node {
    //         let mut a = *a;
    //         a[corner.opposite()] |= b[corner];
    //         Node::new_leaf(a)
    //     }
    //     fn corner_inner_or(a: &Quad<Node>, b: &Quad<Node>, corner: Quadrant) -> Node {
    //         let mut a = *a;
    //         a[corner.opposite()] = equal_node_or(&a[corner.opposite()], &b[corner]);
    //         Node::new_inner(a)
    //     }
    //     fn large_corner_inner_or(a: &Quad<Node>, b: &Node, corner: Quadrant) -> Node {
    //         let mut a = *a;
    //         a[corner.opposite()] = large_corner_node_or(&a[corner.opposite()], b, corner);
    //         Node::new_inner(a)
    //     }
    //     fn large_corner_node_or(a: &Node, b: &Node, corner: Quadrant) -> Node {
    //         if a.depth() == b.depth() {
    //             match (a.depth_quad(), b.depth_quad()) {
    //                 (DepthQuad::Leaf(a), DepthQuad::Leaf(b)) => corner_leaf_or(a, b),
    //                 (DepthQuad::Inner(_, a), DepthQuad::Inner(_, b)) => corner_inner_or(a, b),
    //             }
    //         } else {
    //             large_corner_inner_or(a.inner().unwrap(), b, corner)
    //         }
    //     }
    //     fn corner_node_or(a: &Node, b: &Node, corner: Quadrant) -> Node {
    //         if a.depth() == b.depth() {
    //             equal_node_or(a, b)
    //         } else {
    //             corner_inner_or(a.inner().unwrap(), b, corner)
    //         }
    //     }
    //     fn node_or(a: &Node, b: &Node) -> Node {}
    //
    //     fn center_leaf_or(a: Quad<&Quad<State>>, b: &Quad<State>) -> Node {
    //         Node::new_inner(a.index_map(|i, a| {
    //             let mut a = *a;
    //             a[i.opposite()] |= b[i];
    //             Node::new_leaf(a)
    //         }))
    //     }
    //     fn center_inner_or(a: Quad<&Quad<Node>>, b: &Quad<Node>) -> Node {
    //         Node::new_inner(a.index_map(|i, a| {
    //             // unnecessarilly clones a[i.opposite()], but not sure if that can expressed easilly
    //             let mut a = a.clone();
    //             a[i.opposite()] = equal_node_or(&a[i.opposite()], &b[i]);
    //             Node::new_inner(a)
    //         }))
    //     }
    //     fn center_node_or(a: &Node, b: &Node) -> Node {
    //         match (a.inner().unwrap().children(), b.depth_quad()) {
    //             (DepthQuad::Leaf(a), DepthQuad::Leaf(b)) => center_leaf_or(a, b),
    //             (DepthQuad::Inner(_, a), DepthQuad::Inner(_, b)) => center_inner_or(a, b),
    //         }
    //     }
    //     fn large_leaf_or(a: Quad<&Quad<State>>, b: &Node) -> Node {
    //         center_leaf_or(a, b.leaf().unwrap())
    //     }
    //     fn large_inner_or(a: Quad<&Quad<Node>>, b: &Node) -> Node {
    //         Node::new_inner(a.index_map(|i, a| {
    //             // unnecessarilly clones a[i.opposite()], but not sure if that can expressed easilly
    //             let mut a = a.clone();
    //             a[i.opposite()] = large_node_or(&a[i.opposite()], b);
    //             Node::new_inner(a)
    //         }))
    //     }
    //     fn large_node_or(a: &Node, b: &Node) -> Node {
    //         if a.depth() == b.depth() {
    //             match (a.children(), b.depth_quad()) {
    //                 (DepthQuad::Leaf(a), DepthQuad::Leaf(b)) => center_leaf_or(a, b),
    //                 (DepthQuad::Inner(_, a), DepthQuad::Inner(_, b)) => center_inner_or(a, b),
    //             }
    //         } else {
    //         }
    //     }
    //     fn node_or(a: &Node, b: &Node) -> Node {}
    //     fn recurse_large(large: &Node, small: &Node, ctx: &mut NodeCache) -> Node {
    //         match (large.inner().unwrap().children(), small.depth_quad()) {
    //             (DepthQuad::Leaf(a), DepthQuad::Leaf(b)) => {
    //                 Node::new_inner(a.zip(b).index_map(|quadrant, (a, b)| {
    //                     let mut a = a.clone();
    //                     a[quadrant.opposite()] |= b[quadrant];
    //                     Node::new_leaf(a)
    //                 }))
    //             }
    //         }
    //     }
    //     fn recurse_equal(a: &DepthQuad<Block, Node>, b: &DepthQuad<Block, Node>) -> Node {
    //         match (a, b) {
    //             (DepthQuad::Leaf(a), DepthQuad::Leaf(b)) => {
    //                 Node::new_leaf(a.zip_map(*b, |a, b| a | b))
    //             }
    //             (DepthQuad::Inner(depth, a), DepthQuad::Inner(_, b)) => Node::new_depth_inner(
    //                 *depth,
    //                 a.as_ref().zip_map(b.as_ref(), |a, b| {
    //                     recurse_equal(a.depth_quad(), b.depth_quad())
    //                 }),
    //             ),
    //         }
    //     }
    //     let (large, small) = if self.depth() >= other.depth() {
    //         (self, other)
    //     } else {
    //         (other, self)
    //     };
    // }
    //
    // pub fn offset(&self, amount: Pos, ctx: &mut NodeCache) -> Self {
    //     return if let Some((depth_quad, offset)) = self.bounding_quad() {
    //         let actual_amount = amount + offset;
    //         let small_amount = actual_amount.map(|v| v.rem_euclid(depth_quad.half_width()));
    //         let quad = small_offset(depth_quad, small_amount, ctx);
    //         let result = large_offset(quad, actual_amount - small_amount, ctx);
    //         ctx.inner(result)
    //     } else {
    //         // self is empty
    //         self.clone()
    //     };
    //
    //     fn select<T>(quad: Quad<T>, x: bool, y: bool) -> T {
    //         if x {
    //             if y {
    //                 quad.se
    //             } else {
    //                 quad.ne
    //             }
    //         } else {
    //             if y {
    //                 quad.sw
    //             } else {
    //                 quad.nw
    //             }
    //         }
    //     }
    //
    //     fn small_offset(
    //         depth_quad: &DepthQuad<Block, Node>,
    //         amount: Pos,
    //         ctx: &mut NodeCache,
    //     ) -> Quad<Node> {
    //         match depth_quad {
    //             DepthQuad::Leaf(leaf) => {
    //                 assert!(amount.x == 0);
    //                 assert!(amount.y == 0);
    //                 DepthQuad::Leaf(leaf)
    //             }
    //             DepthQuad::Inner(depth, inner) => {
    //                 let half_width = 1_u64 << depth.get();
    //             }
    //         }
    //     }
    //
    //     fn large_offset(quad: Quad<Node>, amount: Pos, ctx: &mut NodeCache) -> Quad<Node> {}
    // }
}

// not in population.rs since it is cached internally
impl Population for Node {
    fn population(&self) -> u64 {
        self.as_ref().1
    }
    fn is_empty(&self) -> bool {
        self.as_ref().1 == 0
    }
}

impl DepthQuad<Block, Node> {
    fn validate_depth(&self) {
        if let Self::Inner(depth, inner) = self {
            inner.iter().for_each(|node| {
                assert_eq!(depth.get() - 1, node.depth());
            });
        }
    }
}
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.as_ref(), other.as_ref())
    }
}
impl Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.as_ref(), state)
    }
}
impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node( {:p} )", self.as_ref())
    }
}

#[derive(Clone)]
pub struct WeakNode(Weak<NodeData>);
impl Node {
    pub fn weak(&self) -> WeakNode {
        WeakNode(Rc::downgrade(&self.0))
    }
}
impl WeakNode {
    pub fn strong(&self) -> Option<Node> {
        self.0.upgrade().map(Node)
    }
}
impl std::fmt::Debug for WeakNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WeakNode( {:p} )", self.0.as_ptr())
    }
}
impl weak_table::traits::WeakElement for WeakNode {
    type Strong = Node;

    fn new(view: &Self::Strong) -> Self {
        view.weak()
    }
    fn view(&self) -> Option<Self::Strong> {
        self.strong()
    }
    fn is_expired(&self) -> bool {
        self.0.is_expired()
    }
    fn clone(view: &Self::Strong) -> Self::Strong {
        view.clone()
    }
}
impl weak_table::traits::WeakKey for WeakNode {
    type Key = DepthQuad<Block, Node>;

    fn with_key<F, R>(view: &Self::Strong, f: F) -> R
    where
        F: FnOnce(&Self::Key) -> R,
    {
        f(view.depth_quad())
    }
}

// impl Node {
//     pub fn get(&self, pos: &Pos) -> bool {
//         let half_width = self.half_width();
//         if pos.x >= half_width || pos.y >= half_width || pos.x < -half_width || pos.y < -half_width
//         {
//             return false;
//         }
//         let q = Quadrant::from_pos(&pos);
//         match self.depth_quad() {
//             DepthQuad::Leaf(leaf) => {}
//             DepthQuad::Inner(_, _) => todo!(),
//         }
//     }
// }
