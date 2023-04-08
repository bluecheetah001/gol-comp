use lru::LruCache;
use std::cell::RefCell;
use std::num::{NonZeroU64, NonZeroUsize};
use tracing::{trace, trace_span};

use crate::{Block, DepthQuad, Node, Population, Quad};

const LRU_CACHE_SIZE: NonZeroUsize = NonZeroUsize::new(1 << 24).unwrap();

// TODO may want to store Either<Node,Block> results so the smallest cache is 16x16 instead of 32x32
//      xor directly implement Quad<Quad<Block>>::step_center()
struct StepCache {
    lru: LruCache<(Node, NonZeroU64), Node>,
    // it's a bit weird to store this 'globally' and periodicaly reset them
    // but it means it won't be hard to modify reporting to dump this once a second or every 100 calls ect
    // and with a bit of cleanup could track more interesting stats (like breaking this down per depth )
    hit: u32,
    miss: u32,
}
impl StepCache {
    fn new() -> Self {
        Self {
            lru: LruCache::new(LRU_CACHE_SIZE),
            hit: 0,
            miss: 0,
        }
    }
}
thread_local! {
    static STEP_CACHE: RefCell<StepCache> = RefCell::new(StepCache::new());
}

/// depth of 0 is a 16x16 area and can conceptually step 4 times
/// but a node can't represent a 8x8 area
/// so max steps immediately jumps to 8 for a depth of 1
fn depth_to_max_steps(depth: u8) -> u64 {
    if depth == 0 {
        0
    } else {
        1 << (Block::WIDTH_LOG2 - 1 + depth)
    }
}
fn steps_to_min_depth(steps: NonZeroU64) -> u8 {
    if steps.get() <= Block::WIDTH {
        1
    } else {
        #[allow(clippy::cast_possible_truncation)] // max from ilog2 is 32
        let floor = steps.ilog2() as u8 - (Block::WIDTH_LOG2 - 1);
        if steps.is_power_of_two() {
            floor
        } else {
            floor + 1
        }
    }
}

// buffer logic

impl Node {
    pub fn step(&self, steps: u64) -> Node {
        match NonZeroU64::new(steps) {
            None => self.clone(),
            Some(steps) => {
                // step will reduce the area of the node, but also the unbuffered node will grow up to the same amount
                // so we need 2 buffer nodes that are both >= min_depth
                // so we unbuffer given node so it is >= min_depth - 1
                let min_depth = steps_to_min_depth(steps);
                let depth = self.unbufferd_depth(min_depth - 1) + 2;

                let _span = trace_span!("step", depth, steps).entered();

                let root = self.center_at_depth(depth);
                STEP_CACHE.with_borrow_mut(|step_cache| {
                    let result = root.step_center(steps, step_cache);

                    trace!(step_cache.hit, step_cache.miss, "cache_perf");
                    step_cache.hit = 0;
                    step_cache.miss = 0;

                    result
                })
            }
        }
    }
    // find the smallest depth where the node is unbuffered, maxed with target_depth
    fn unbufferd_depth(&self, target_depth: u8) -> u8 {
        fn unbufferd_depth_inner(inner: Quad<&Node>, target_depth: u8) -> u8 {
            let child_depth = inner.nw.depth();
            if child_depth < target_depth {
                target_depth
            } else {
                match inner.children() {
                    DepthQuad::Leaf(leaf) if leaf.is_buffered() => child_depth,
                    DepthQuad::Inner(_, inner) if inner.is_buffered() => {
                        unbufferd_depth_inner(inner.center(), target_depth)
                    }
                    _ => child_depth + 1,
                }
            }
        }

        let node_depth = self.depth();
        if node_depth <= target_depth {
            target_depth
        } else {
            unbufferd_depth_inner(self.inner().unwrap().as_ref(), target_depth)
        }
    }
}
impl<T> Quad<&Quad<T>>
where
    T: Population,
{
    fn is_buffered(&self) -> bool {
        [
            &self.nw.nw,
            &self.nw.ne,
            &self.nw.sw,
            &self.ne.nw,
            &self.ne.ne,
            &self.ne.se,
            &self.sw.nw,
            &self.sw.sw,
            &self.sw.se,
            &self.se.ne,
            &self.se.sw,
            &self.se.se,
        ]
        .into_iter()
        .all(T::is_empty)
    }
}

// recurse logic

impl Node {
    fn step_center(self, steps: NonZeroU64, step_cache: &mut StepCache) -> Node {
        let key = (self, steps);
        match step_cache.lru.get(&key) {
            None => {
                step_cache.miss += 1;
                let result = key.0.step_center_impl(steps, step_cache);
                step_cache.lru.put(key, result.clone());
                result
            }
            Some(result) => {
                step_cache.hit += 1;
                result.clone()
            }
        }
    }
    fn step_center_impl(&self, steps: NonZeroU64, step_cache: &mut StepCache) -> Node {
        let max_steps = depth_to_max_steps(self.depth());
        debug_assert!(steps.get() <= max_steps);
        let first_half_steps = steps.get().saturating_sub(max_steps / 2);
        let second_half_steps = steps.get() - first_half_steps;
        match self.inner().unwrap().as_ref().children() {
            DepthQuad::Leaf(leaf) => leaf
                .copied()
                .overlaps_hood()
                .map(|quad| quad.step_center(first_half_steps))
                .overlaps_quad()
                .map(|quad| quad.step_center(second_half_steps))
                .into(),
            DepthQuad::Inner(_, inner) => inner
                .cloned()
                .overlaps_hood()
                .map(|quad| quad.step_center(first_half_steps, step_cache))
                .overlaps_quad()
                .map(|quad| quad.step_center(second_half_steps, step_cache))
                .into(),
        }
    }
}
impl Quad<Node> {
    fn step_center(self, steps: u64, step_cache: &mut StepCache) -> Node {
        match NonZeroU64::new(steps) {
            None => self.as_ref().center().into(),
            Some(steps) => Node::from(self).step_center(steps, step_cache),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Hood<T> {
    nw: T,
    n: T,
    ne: T,
    w: T,
    c: T,
    e: T,
    sw: T,
    s: T,
    se: T,
}
impl<T> Hood<T> {
    fn map<U>(self, mut f: impl FnMut(T) -> U) -> Hood<U> {
        Hood {
            nw: f(self.nw),
            n: f(self.n),
            ne: f(self.ne),
            w: f(self.w),
            c: f(self.c),
            e: f(self.e),
            sw: f(self.sw),
            s: f(self.s),
            se: f(self.se),
        }
    }
}

impl<T> Quad<Quad<T>>
where
    T: Clone,
{
    #[allow(clippy::many_single_char_names)]
    fn overlaps_hood(self) -> Hood<Quad<T>> {
        let n = Quad {
            nw: self.nw.ne.clone(),
            ne: self.ne.nw.clone(),
            sw: self.nw.se.clone(),
            se: self.ne.sw.clone(),
        };
        let w = Quad {
            nw: self.nw.sw.clone(),
            ne: self.nw.se.clone(),
            sw: self.sw.nw.clone(),
            se: self.sw.ne.clone(),
        };
        let c = self.as_ref().center().cloned();
        let e = Quad {
            nw: self.ne.sw.clone(),
            ne: self.ne.se.clone(),
            sw: self.se.nw.clone(),
            se: self.se.ne.clone(),
        };
        let s = Quad {
            nw: self.sw.ne.clone(),
            ne: self.se.nw.clone(),
            sw: self.sw.se.clone(),
            se: self.se.sw.clone(),
        };
        let Self { nw, ne, sw, se } = self;
        Hood {
            nw,
            n,
            ne,
            w,
            c,
            e,
            sw,
            s,
            se,
        }
    }
}
impl<T> Hood<T>
where
    T: Clone,
{
    fn overlaps_quad(self) -> Quad<Quad<T>> {
        Quad {
            nw: Quad {
                nw: self.nw,
                ne: self.n.clone(),
                sw: self.w.clone(),
                se: self.c.clone(),
            },
            ne: Quad {
                nw: self.n,
                ne: self.ne,
                sw: self.c.clone(),
                se: self.e.clone(),
            },
            sw: Quad {
                nw: self.w,
                ne: self.c.clone(),
                sw: self.sw,
                se: self.s.clone(),
            },
            se: Quad {
                nw: self.c,
                ne: self.e,
                sw: self.s,
                se: self.se,
            },
        }
    }
}

// base logic

impl Quad<Block> {
    fn step_center(&self, steps: u64) -> Block {
        debug_assert!(steps <= 4);
        // convert 4 8x8 blocks into 4 4x16 blocks
        let mut rows = [
            shape_north(self.nw.to_rows(), self.ne.to_rows()),
            shape_south(self.nw.to_rows(), self.ne.to_rows()),
            shape_north(self.sw.to_rows(), self.se.to_rows()),
            shape_south(self.sw.to_rows(), self.se.to_rows()),
        ];
        // TODO could optimize by dropping indicies that are no longer needed
        for _ in 0..steps {
            step_rows_once(&mut rows);
        }
        // convert back into an 8x8 block
        Block::from_rows(unshape_center(rows[1], rows[2]))
    }
}
#[allow(clippy::similar_names)]
fn shape_north(w: u64, e: u64) -> u64 {
    let r0w = w & 0xff_00_00_00_00_00_00_00;
    let r0e = (e >> 8) & 0x00_ff_00_00_00_00_00_00;
    let r1w = (w >> 8) & 0x00_00_ff_00_00_00_00_00;
    let r1e = (e >> 16) & 0x00_00_00_ff_00_00_00_00;
    let r2w = (w >> 16) & 0x00_00_00_00_ff_00_00_00;
    let r2e = (e >> 24) & 0x00_00_00_00_00_ff_00_00;
    let r3w = (w >> 24) & 0x00_00_00_00_00_00_ff_00;
    let r3e = (e >> 32) & 0x00_00_00_00_00_00_00_ff;
    r0w | r0e | r1w | r1e | r2w | r2e | r3w | r3e
}
#[allow(clippy::similar_names)]
fn shape_south(w: u64, e: u64) -> u64 {
    let r0w = (w << 32) & 0xff_00_00_00_00_00_00_00;
    let r0e = (e << 24) & 0x00_ff_00_00_00_00_00_00;
    let r1w = (w << 24) & 0x00_00_ff_00_00_00_00_00;
    let r1e = (e << 16) & 0x00_00_00_ff_00_00_00_00;
    let r2w = (w << 16) & 0x00_00_00_00_ff_00_00_00;
    let r2e = (e << 8) & 0x00_00_00_00_00_ff_00_00;
    let r3w = (w << 8) & 0x00_00_00_00_00_00_ff_00;
    let r3e = e & 0x00_00_00_00_00_00_00_ff;
    r0w | r0e | r1w | r1e | r2w | r2e | r3w | r3e
}
fn unshape_center(n: u64, s: u64) -> u64 {
    let r0 = (n << 4) & 0xff_00_00_00_00_00_00_00;
    let r1 = (n << 12) & 0x00_ff_00_00_00_00_00_00;
    let r2 = (n << 20) & 0x00_00_ff_00_00_00_00_00;
    let r3 = (n << 28) & 0x00_00_00_ff_00_00_00_00;
    let r4 = (s >> 28) & 0x00_00_00_00_ff_00_00_00;
    let r5 = (s >> 20) & 0x00_00_00_00_00_ff_00_00;
    let r6 = (s >> 12) & 0x00_00_00_00_00_00_ff_00;
    let r7 = (s >> 4) & 0x00_00_00_00_00_00_00_ff;
    r0 | r1 | r2 | r3 | r4 | r5 | r6 | r7
}

fn step_rows_once(rows: &mut [u64; 4]) {
    // isn't necessarilly very optimized, but it will take a lot of staring at generated asm to find something better
    // in particular it may be that shifting data up 1 row (left by 16 bits) each call can both use less registers (only need 2 values for each call to step_row)
    // and omiting calls that produce unused values: only need rows 1,2 for the last step; or even fewer, but more complicated, when shifting data
    let step_row_shift =
        |prev, row, next| step_row((prev << 48) | (row >> 16), row, (row << 16) | (next >> 48));
    *rows = [
        step_row_shift(0, rows[0], rows[1]),
        step_row_shift(rows[0], rows[1], rows[2]),
        step_row_shift(rows[1], rows[2], rows[3]),
        step_row_shift(rows[2], rows[3], 0),
    ];
}
fn step_row(above: u64, row: u64, below: u64) -> u64 {
    // compute a bitwise addition of 3 values (as 2 separate results)
    let bit_sum = |a, b, c| (a ^ b ^ c, a & b | a & c | b & c);

    let (i0, i1) = bit_sum(above, row, below); // vertical sum i0=1,3 i1=2,3
    let (a0, a1) = bit_sum(i0 << 1, above ^ below, i0 >> 1); // horizontal sum of 1 or 3 for each column a0=1,3,5,7 a1=2,3,4,5,6,7
    let (b0, b1) = bit_sum(i1 << 1, above & below, i1 >> 1); // horizontal sum of 2 or 3 for each column b0=2,3,6,7,8 b2=4,5,6,7,8

    // t | cols  | a b   a0a1b0b1
    // 0 | 0 0 0 | 0 0 || 0 0 0 0
    // 1 | 0 0 1 | 1 0 || 1 0 0 0
    // 2 | 0 0 2 | 0 1 || 0 0 1 0
    // 2 | 0 1 1 | 2 0 || 0 1 0 0
    // 3 | 0 0 3 | 1 1 || 1 0 1 0
    // 3 | 0 1 2 | 1 1 || 1 0 1 0
    // 3 | 1 1 1 | 3 0 || 1 1 0 0
    // 4 | 0 1 3 | 2 1 || 0 1 1 0
    // 4 | 0 2 2 | 0 2 || 0 0 0 1
    // 4 | 1 1 2 | 2 1 || 0 1 1 0
    // 5 | 0 2 3 | 1 2 || 1 0 0 1
    // 5 | 1 1 3 | 3 1 || 1 1 1 0
    // 5 | 1 2 2 | 1 2 || 1 0 0 1
    // 6 | 0 3 3 | 2 2 || 0 1 0 1
    // 6 | 1 2 3 | 2 2 || 0 1 0 1
    // 6 | 2 2 2 | 0 3 || 0 0 1 1
    // 7 | 1 3 3 | 3 2 || 1 1 0 1
    // 7 | 2 2 3 | 1 3 || 1 0 1 1
    // 8 | 2 3 3 | 2 3 || 0 1 1 1
    // a0 = odd
    // a1 ^ b0 = 2,3,6,7
    // b1 >= 6 (and some from 4 and 5)
    // (row | a0) causes odd neighbors to become alive
    // (a1 ^ b0) & !b1 is 2 or 3 neighbors
    (row | a0) & (a1 ^ b0) & !b1
}

// tests

#[cfg(test)]
#[allow(clippy::many_single_char_names)]
mod tests {
    use crate::{test_block, Block, Node};

    use super::{depth_to_max_steps, steps_to_min_depth};
    use std::num::NonZeroU64;

    #[test]
    fn test_depth_to_max_steps() {
        assert_eq!(depth_to_max_steps(0), 0);
        assert_eq!(depth_to_max_steps(1), 8);
        assert_eq!(depth_to_max_steps(2), 16);
        assert_eq!(depth_to_max_steps(3), 32);
        assert_eq!(depth_to_max_steps(Node::MAX_DEPTH), 1 << 61);
    }
    #[test]
    fn test_steps_to_min_depth() {
        assert_eq!(steps_to_min_depth(NonZeroU64::new(1).unwrap()), 1);
        assert_eq!(steps_to_min_depth(NonZeroU64::new(8).unwrap()), 1);
        assert_eq!(steps_to_min_depth(NonZeroU64::new(9).unwrap()), 2);
        assert_eq!(steps_to_min_depth(NonZeroU64::new(16).unwrap()), 2);
        assert_eq!(steps_to_min_depth(NonZeroU64::new(17).unwrap()), 3);
        assert_eq!(steps_to_min_depth(NonZeroU64::new(32).unwrap()), 3);
        assert_eq!(
            steps_to_min_depth(NonZeroU64::new((1 << 60) + 1).unwrap()),
            Node::MAX_DEPTH
        );
        assert_eq!(
            steps_to_min_depth(NonZeroU64::new(1 << 61).unwrap()),
            Node::MAX_DEPTH
        );
    }

    fn assert_block_step(input: Block, steps: u64, output: Block) {
        assert_eq!(input.expand().step_center(steps), output);
    }

    #[test]
    fn block_still() {
        let a = test_block! {"
            ........
            ........
            ........
            ........
            ........
            ........
            ........
            ........
            "};
        let b = test_block! {"
            ......o.
            .oo..o.o
            .oo...o.
            ........
            ......o.
            oo...o.o
            o.o..o.o
            .o....o.
            "};
        let c = test_block! {"
            .oo.....
            o..o....
            o..o....
            .oo.....
            ......o.
            .....o.o
            ....o..o
            .....oo.
            "};
        assert_block_step(a, 1, a);
        assert_block_step(b, 1, b);
        assert_block_step(c, 1, c);
    }

    #[test]
    fn block_blinker_a() {
        let a = test_block! {"
            ........
            .....ooo
            ....ooo.
            ........
            ........
            ........
            ooo.....
            ........
            "};
        let b = test_block! {"
            ......o.
            ....o..o
            ....o..o
            .....o..
            ........
            .o......
            .o......
            .o......
            "};
        assert_block_step(a, 1, b);
        assert_block_step(a, 2, a);
        assert_block_step(b, 1, a);
        assert_block_step(b, 2, b);
    }

    #[test]
    fn block_blinker_b() {
        let a = test_block! {"
            ..oo....
            ..oo....
            oo......
            oo....oo
            .......o
            ....o.o.
            ...o....
            ...oo...
            "};
        let b = test_block! {"
            ..oo....
            ...o....
            o.......
            oo....oo
            .....o.o
            ........
            ...o.o..
            ...oo...
            "};
        assert_block_step(a, 1, b);
        assert_block_step(a, 2, a);
        assert_block_step(b, 1, a);
        assert_block_step(b, 2, b);
    }

    #[test]
    fn block_jam() {
        let a = test_block! {"
            .....oo.
            ....o..o
            ..o..o.o
            ..o...o.
            ..o.....
            .....o..
            ...oo...
            ........
            "};
        let b = test_block! {"
            .....oo.
            ....o..o
            ...o.o.o
            .ooo..o.
            ........
            ...oo...
            ....o...
            ........
            "};
        let c = test_block! {"
            .....oo.
            ....o..o
            ...o.o.o
            ..ooo.o.
            ....o...
            ...oo...
            ...oo...
            ........
            "};
        assert_block_step(a, 1, b);
        assert_block_step(a, 2, c);
        assert_block_step(b, 1, c);
        assert_block_step(b, 2, a);
        assert_block_step(c, 1, a);
        assert_block_step(c, 2, b);
    }

    #[test]
    fn block_blinker_on_edge() {
        let a = test_block! {"
            o....ooo
            o.......
            o.......
            ........
            ........
            .......o
            .......o
            ooo....o
            "};
        let b = test_block! {"
            ......o.
            oo....o.
            ........
            ........
            ........
            ........
            .o....oo
            .o......
            "};
        let c = test_block! {"
            ........
            ........
            ........
            ........
            ........
            ........
            ........
            ........
            "};
        assert_block_step(a, 1, b);
        assert_block_step(a, 2, a);
        assert_block_step(b, 1, c);
    }

    #[test]
    fn block_glider() {
        let a = test_block! {"
            ........
            ........
            ........
            ...ooo..
            ...o....
            ....o...
            ........
            ........
            "};
        let b = test_block! {"
            ........
            ........
            ....o...
            ...oo...
            ...o.o..
            ........
            ........
            ........
            "};
        let c = test_block! {"
            ........
            ........
            ...oo...
            ...o.o..
            ...o....
            ........
            ........
            ........
            "};
        let d = test_block! {"
            ........
            ........
            ...oo...
            ..oo....
            ....o...
            ........
            ........
            ........
            "};
        let e = test_block! {"
            ........
            ........
            ..ooo...
            ..o.....
            ...o....
            ........
            ........
            ........
            "};
        assert_block_step(a, 1, b);
        assert_block_step(a, 2, c);
        assert_block_step(a, 3, d);
        assert_block_step(a, 4, e);
    }

    #[test]
    fn block_pentadecathlon() {
        let a = test_block! {"
            ........
            ........
            oooooooo
            o.oooo.o
            oooooooo
            ........
            ........
            ........
            "};
        let b = test_block! {"
            ........
            .oooooo.
            o......o
            ........
            o......o
            .oooooo.
            ........
            ........
            "};
        let c = test_block! {"
            ..oooo..
            .oooooo.
            oooooooo
            o......o
            oooooooo
            .oooooo.
            ..oooo..
            ........
            "};
        let d = test_block! {"
            .o....o.
            o......o
            ........
            ........
            ........
            o......o
            .o....o.
            ...oo...
            "};
        let e = test_block! {"
            ........
            o......o
            o......o
            o......o
            o......o
            o......o
            ........
            ........
            "};
        assert_block_step(a, 1, b);
        assert_block_step(a, 2, c);
        assert_block_step(a, 3, d);
        assert_block_step(a, 4, e);
    }
}
