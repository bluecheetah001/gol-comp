use std::cell::RefCell;
use std::num::NonZeroUsize;

use lru::LruCache;

use crate::{Block, DepthQuad, Node, Population, Quad};

const LRU_CACHE_SIZE: NonZeroUsize = NonZeroUsize::new(1 << 8).unwrap();

struct ReduceCache {
    to_depth: LruCache<(Node, u8), Node>,
}
impl ReduceCache {
    fn new() -> Self {
        Self {
            to_depth: LruCache::new(LRU_CACHE_SIZE),
        }
    }
}
thread_local! {
    static REDUCE_CACHE: RefCell<ReduceCache> = RefCell::new(ReduceCache::new());
}

impl Node {
    pub fn reduce_by(&self, amount: u8) -> Self {
        if amount == 0 {
            return self.clone();
        }
        let depth = self.depth();
        REDUCE_CACHE.with_borrow_mut(|reduce_cache| {
            if amount > depth {
                self.center_at_depth(amount).reduce_to(0, reduce_cache)
            } else {
                self.reduce_to(depth - amount, reduce_cache)
            }
        })
    }
    fn reduce_to(&self, depth: u8, reduce_cache: &mut ReduceCache) -> Self {
        // TODO annoying that this needs a clone here rather than just before putting
        // but fixing it requires a more clever LRU implementation (like IndexMap's Equivalent)
        // `step_center` has a similar problem, but is is able to take an owned Node
        let key = (self.clone(), depth);
        match reduce_cache.to_depth.get(&key) {
            None => {
                // reduce_cache.miss += 1;
                let result = key.0.reduce_to_impl(depth, reduce_cache);
                reduce_cache.to_depth.put(key, result.clone());
                result
            }
            Some(result) => {
                // reduce_cache.hit += 1;
                result.clone()
            }
        }
    }
    fn reduce_to_impl(&self, depth: u8, reduce_cache: &mut ReduceCache) -> Self {
        let inner = self.inner().expect("to.depth to be < self.depth").as_ref();
        if depth == 0 {
            inner.map(Node::reduce_to_block).into()
        } else {
            inner
                .map(|node| node.reduce_to(depth - 1, reduce_cache))
                .into()
        }
    }
    // this is still a decent amount of (constant) work, should it be memoized?
    fn reduce_to_block(&self) -> Block {
        match self.depth_quad() {
            DepthQuad::Leaf(leaf) => leaf.reduce_to_block(),
            // TODO for sufficiently large depth this could be more efficient by looping over 64 children^3() and calling is_empty()
            DepthQuad::Inner(_, inner) => match inner.as_ref().children() {
                DepthQuad::Leaf(leaf) => leaf.map(Quad::reduce_to_block).reduce_to_block(),
                DepthQuad::Inner(_, inner) => {
                    let inner = inner.map(Quad::reduce_to_16);
                    let nw = inner.nw << (4 * 8 + 4);
                    let ne = inner.ne << (4 * 8);
                    let sw = inner.sw << 4;
                    let se = inner.se;
                    Block::from_rows(nw | ne | sw | se)
                }
            },
        }
    }
    fn reduce_to_4(&self) -> u64 {
        let empty = match self.depth_quad() {
            crate::DepthQuad::Leaf(leaf) => leaf.as_ref().map(Block::is_empty),
            crate::DepthQuad::Inner(_, inner) => inner.as_ref().map(Node::is_empty),
        };
        let mut rows = 0_u64;
        if !empty.nw {
            rows |= 1 << 9;
        }
        if !empty.ne {
            rows |= 1 << 8;
        }
        if !empty.sw {
            rows |= 2;
        }
        if !empty.se {
            rows |= 1;
        }
        rows
    }
}
impl Quad<Node> {
    fn reduce_to_16(&self) -> u64 {
        let quad = self.as_ref().map(Node::reduce_to_4);
        let nw = quad.nw << (2 * 8 + 2);
        let ne = quad.ne << (2 * 8);
        let sw = quad.sw << 2;
        let se = quad.se;
        nw | ne | sw | se
    }
}
impl Quad<Block> {
    fn reduce_to_block(&self) -> Block {
        fn zoom_out_h(w: u64, e: u64) -> u64 {
            let w = w | (w << 1);
            let e = e | (e << 1);
            let c0 = w & 0x80_80_80_80_80_80_80_80;
            let c1 = (w << 1) & 0x40_40_40_40_40_40_40_40;
            let c2 = (w << 2) & 0x20_20_20_20_20_20_20_20;
            let c3 = (w << 3) & 0x10_10_10_10_10_10_10_10;
            let c4 = (e >> 4) & 0x08_08_08_08_08_08_08_08;
            let c5 = (e >> 3) & 0x04_04_04_04_04_04_04_04;
            let c6 = (e >> 2) & 0x02_02_02_02_02_02_02_02;
            let c7 = (e >> 1) & 0x01_01_01_01_01_01_01_01;
            c0 | c1 | c2 | c3 | c4 | c5 | c6 | c7
        }
        fn zoom_out_v(n: u64, s: u64) -> u64 {
            let n = n | (n << 8);
            let s = s | (s << 8);
            let r0 = n & 0xff_00_00_00_00_00_00_00;
            let r1 = (n << 8) & 0x00_ff_00_00_00_00_00_00;
            let r2 = (n << 16) & 0x00_00_ff_00_00_00_00_00;
            let r3 = (n << 24) & 0x00_00_00_ff_00_00_00_00;
            let r4 = (s >> 32) & 0x00_00_00_00_ff_00_00_00;
            let r5 = (s >> 24) & 0x00_00_00_00_00_ff_00_00;
            let r6 = (s >> 16) & 0x00_00_00_00_00_00_ff_00;
            let r7 = (s >> 8) & 0x00_00_00_00_00_00_00_ff;
            r0 | r1 | r2 | r3 | r4 | r5 | r6 | r7
        }
        let Quad { nw, ne, sw, se } = self.map(Block::to_rows);
        Block::from_rows(zoom_out_v(zoom_out_h(nw, ne), zoom_out_h(sw, se)))
    }
}
#[cfg(test)]
mod test {
    use crate::{Block, Node};

    #[test]
    fn corners() {
        let node0 = Node::new(
            Block::from_rows(0x80_00_00_00_00_00_00_00),
            Block::from_rows(0x01_00_00_00_00_00_00_00),
            Block::from_rows(0x00_00_00_00_00_00_00_80),
            Block::from_rows(0x00_00_00_00_00_00_00_01),
        );
        let node1 = Node::new(
            Block::from_rows(0x00_00_00_00_08_00_00_00),
            Block::from_rows(0x00_00_00_00_10_00_00_00),
            Block::from_rows(0x00_00_00_08_00_00_00_00),
            Block::from_rows(0x00_00_00_10_00_00_00_00),
        );
        let node2 = Node::new(
            Block::from_rows(0x00_00_00_00_00_00_02_00),
            Block::from_rows(0x00_00_00_00_00_00_40_00),
            Block::from_rows(0x00_02_00_00_00_00_00_00),
            Block::from_rows(0x00_40_00_00_00_00_00_00),
        );
        let node3 = Node::new(
            Block::from_rows(0x00_00_00_00_00_00_00_01),
            Block::from_rows(0x00_00_00_00_00_00_00_80),
            Block::from_rows(0x01_00_00_00_00_00_00_00),
            Block::from_rows(0x80_00_00_00_00_00_00_00),
        );
        assert_eq!(node0.reduce_by(0), node0);
        assert_eq!(node0.reduce_by(1), node1);
        assert_eq!(node0.reduce_by(2), node2);
        assert_eq!(node0.reduce_by(3), node3);
        assert_eq!(node0.reduce_by(4), node3);
    }

    #[test]
    fn basic() {
        let node0_a = Node::new(
            Block::from_rows(0x80_40_20_10_08_04_02_01),
            Block::from_rows(0x01_02_04_08_10_20_40_80),
            Block::from_rows(0x80_40_20_10_08_04_02_01),
            Block::from_rows(0x01_02_04_08_10_20_40_80),
        );
        let node0 = Node::new(node0_a.clone(), node0_a.clone(), node0_a.clone(), node0_a);
        let block1_a = Block::from_rows(0x81_42_24_18_81_42_24_18);
        let node1 = Node::new(block1_a, block1_a, block1_a, block1_a);
        assert_eq!(node0.reduce_by(0), node0);
        assert_eq!(node0.reduce_by(1), node1);
    }
}
