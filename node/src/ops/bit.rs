use std::cell::RefCell;

use crate::ops::cache::Cache;
use crate::{Block, Node};

pub const OP_CACHE_SIZE: usize = 1 << 8;
type OpCache = Cache<(Node, Node), Node>;

thread_local! {
    static OR_CACHE: RefCell<OpCache> = RefCell::new(Cache::new("Node::or", OP_CACHE_SIZE));
    static XOR_CACHE: RefCell<OpCache> = RefCell::new(Cache::new("Node::xor", OP_CACHE_SIZE));
    static AND_CACHE: RefCell<OpCache> = RefCell::new(Cache::new("Node::and", OP_CACHE_SIZE));
    static AND_NOT_CACHE: RefCell<OpCache> = RefCell::new(Cache::new("Node::and_not", OP_CACHE_SIZE));
}

// could override & | ^, but fns are clearer since they allow `and_not` even though `not` isn't possible (don't support infinite field of alive)
impl Node {
    fn zip_map<F: Copy + Fn(Block, Block) -> Block>(
        self,
        rhs: Node,
        op: F,
        cache: &mut OpCache,
    ) -> Node {
        cache.get_or_insert((self, rhs), |(lhs, rhs), cache| {
            lhs.zip_map_impl(rhs, op, cache)
        })
    }
    fn zip_map_impl<F: Copy + Fn(Block, Block) -> Block>(
        &self,
        rhs: &Node,
        op: F,
        cache: &mut OpCache,
    ) -> Node {
        match (self.depth_quad(), rhs.depth_quad()) {
            (crate::DepthQuad::Leaf(lhs), crate::DepthQuad::Leaf(rhs)) => {
                lhs.zip_map(*rhs, op).into()
            }
            (crate::DepthQuad::Inner(depth, lhs), crate::DepthQuad::Inner(_, rhs)) => {
                Node::new_depth_inner(
                    *depth,
                    lhs.as_ref().zip_map(rhs.as_ref(), move |l, r| {
                        l.clone().zip_map(r.clone(), op, cache)
                    }),
                )
            }
            _ => panic!("inconsistent depth"),
        }
    }

    pub fn or(&self, rhs: &Node) -> Node {
        let depth = self.depth().max(rhs.depth());
        let lhs = self.center_at_depth(depth);
        let rhs = rhs.center_at_depth(depth);
        Cache::with(&OR_CACHE, |cache| lhs.zip_map(rhs, Block::or, cache))
    }
    pub fn xor(&self, rhs: &Node) -> Node {
        let depth = self.depth().max(rhs.depth());
        let lhs = self.center_at_depth(depth);
        let rhs = rhs.center_at_depth(depth);
        Cache::with(&XOR_CACHE, |cache| lhs.zip_map(rhs, Block::xor, cache))
    }
    pub fn and(&self, rhs: &Node) -> Node {
        let depth = self.depth().min(rhs.depth());
        let lhs = self.center_at_depth(depth);
        let rhs = rhs.center_at_depth(depth);
        Cache::with(&AND_CACHE, |cache| lhs.zip_map(rhs, Block::and, cache))
    }
    pub fn and_not(&self, rhs: &Node) -> Node {
        let depth = self.depth().max(rhs.depth());
        let lhs = self.center_at_depth(depth);
        let rhs = rhs.center_at_depth(depth);
        Cache::with(&AND_NOT_CACHE, |cache| {
            lhs.zip_map(rhs, Block::and_not, cache)
        })
    }
}

impl Block {
    pub fn or(self, rhs: Self) -> Self {
        Self::from_rows(self.to_rows() | rhs.to_rows())
    }
    pub fn and(self, rhs: Self) -> Self {
        Self::from_rows(self.to_rows() & rhs.to_rows())
    }
    pub fn and_not(self, rhs: Self) -> Self {
        Self::from_rows(self.to_rows() & !rhs.to_rows())
    }
    pub fn xor(self, rhs: Self) -> Self {
        Self::from_rows(self.to_rows() ^ rhs.to_rows())
    }
}

// TODO testing!

#[cfg(test)]
mod tests {
    use crate::{Block, Node};

    #[test]
    fn block_ops() {
        let a = Block::from_rows(0xff_0f_33_55_12_34_56_78);
        let b = Block::from_rows(0x87_65_43_21_55_33_0f_ff);
        assert_eq!(a.or(b), Block::from_rows(0xff_6f_73_75_57_37_5f_ff));
        assert_eq!(a.xor(b), Block::from_rows(0x78_6a_70_74_47_07_59_87));
        assert_eq!(a.and(b), Block::from_rows(0x87_05_03_01_10_30_06_78));
        assert_eq!(a.and_not(b), Block::from_rows(0x78_0a_30_54_02_04_50_00));
        assert_eq!(b.and_not(a), Block::from_rows(0x00_60_40_20_45_03_09_87));
    }

    #[test]
    fn node_ops() {
        let a = Node::new(
            Block::from_rows(0xff_0f_33_55_12_34_56_78),
            Block::from_rows(0x03_06_0c_1f_30_60_c0_00),
            Block::from_rows(0xff_81_81_81_81_81_81_ff),
            Block::from_rows(0x00_00_00_00_00_00_00_00),
        );
        let b = Node::new(
            Block::from_rows(0x87_65_43_21_55_33_0f_ff),
            Block::from_rows(0xc0_60_30_1f_0c_06_03_00),
            Block::from_rows(0x18_24_42_81_81_42_24_18),
            Block::from_rows(0xff_ff_ff_ff_ff_ff_ff_ff),
        );
        assert_eq!(a.or(&b), Node::empty(0));
        assert_eq!(a.xor(&b), Node::empty(0));
        assert_eq!(a.and(&b), Node::empty(0));
        assert_eq!(a.and_not(&b), Node::empty(0));
        assert_eq!(b.and_not(&a), Node::empty(0));
    }
}
