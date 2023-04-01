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
