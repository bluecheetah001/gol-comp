use std::hash::Hash;
use std::num::{NonZeroU64, NonZeroU8, NonZeroUsize};
use std::rc::{Rc, Weak};

use lru::LruCache;
use weak_table::WeakHashSet;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct State(bool);
impl State {
    fn from_alive(alive: bool) -> Self {
        Self(alive)
    }
    fn from_dead(dead: bool) -> Self {
        Self(!dead)
    }
    fn dead() -> Self {
        Self(false)
    }
    fn alive() -> Self {
        Self(true)
    }
    fn step(
        nw: Self,
        n: Self,
        ne: Self,
        w: Self,
        c: Self,
        e: Self,
        sw: Self,
        s: Self,
        se: Self,
    ) -> Self {
        let total = nw.as_u8()
            + n.as_u8()
            + ne.as_u8()
            + w.as_u8()
            + e.as_u8()
            + sw.as_u8()
            + s.as_u8()
            + se.as_u8();
        let alive = if c.0 {
            total == 2 || total == 3
        } else {
            total == 3
        };
        Self(alive)
    }
    fn is_alive(&self) -> bool {
        self.0
    }
    fn is_dead(&self) -> bool {
        !self.0
    }
    fn as_u8(&self) -> u8 {
        self.0 as u8
    }
}

const MAX_DEPTH: u8 = 63;
const MAX_STEPS: u64 = 1 << 62;
const LRU_CACHE_SIZE: NonZeroUsize = NonZeroUsize::new(1 << 24).unwrap();

fn depth_to_max_steps(depth: u8) -> u64 {
    assert!(
        depth <= MAX_DEPTH,
        "depth {} must be <= {}",
        depth,
        MAX_DEPTH
    );
    if depth == 0 {
        0
    } else {
        1 << (depth - 1)
    }
}
fn steps_to_min_depth(steps: NonZeroU64) -> u8 {
    assert!(
        steps.get() <= MAX_STEPS,
        "steps {} must be <= {}",
        steps,
        MAX_STEPS
    );
    if steps.is_power_of_two() {
        steps.ilog2() as u8 + 1
    } else {
        steps.ilog2() as u8 + 2
    }
}
// the absolute limit on width is 2^64, which is not representable
fn depth_to_half_width(depth: u8) -> u64 {
    assert!(
        depth <= MAX_DEPTH,
        "depth {} must be <= {}",
        depth,
        MAX_DEPTH
    );
    1 << depth
}
fn half_width_to_depth(half_width: u64) -> u8 {
    assert!(
        half_width.is_power_of_two(),
        "half_width {} must be a power of 2",
        half_width
    );
    half_width.ilog2() as u8
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum Node {
    Leaf(LeafNode),
    Inner(NonZeroU8, InnerNode),
}
impl Node {
    fn depth(&self) -> u8 {
        match self {
            Self::Leaf(_) => 0,
            Self::Inner(depth, _) => depth.get(),
        }
    }
    fn max_steps(&self) -> u64 {
        depth_to_max_steps(self.depth())
    }
    fn half_width(&self) -> u64 {
        depth_to_half_width(self.depth())
    }

    fn leaf(&self) -> Option<&LeafNode> {
        match self {
            Self::Leaf(leaf) => Some(leaf),
            Self::Inner(_, _) => None,
        }
    }
    fn inner(&self) -> Option<&InnerNode> {
        match self {
            Self::Leaf(_) => None,
            Self::Inner(_, inner) => Some(inner),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct LeafNode {
    nw: State,
    ne: State,
    sw: State,
    se: State,
}
impl LeafNode {
    fn new(nw: State, ne: State, sw: State, se: State) -> Self {
        Self { nw, ne, sw, se }
    }
}

#[derive(Clone, Eq)]
struct InnerNode {
    nw: Rc<Node>,
    ne: Rc<Node>,
    sw: Rc<Node>,
    se: Rc<Node>,
}
impl InnerNode {
    fn new(nw: Rc<Node>, ne: Rc<Node>, sw: Rc<Node>, se: Rc<Node>) -> Self {
        Self { nw, ne, sw, se }
    }
}
impl PartialEq for InnerNode {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.nw, &other.nw)
            && Rc::ptr_eq(&self.ne, &other.ne)
            && Rc::ptr_eq(&self.sw, &other.sw)
            && Rc::ptr_eq(&self.se, &other.se)
    }
}
impl std::hash::Hash for InnerNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.nw.as_ref(), state);
        std::ptr::hash(self.ne.as_ref(), state);
        std::ptr::hash(self.sw.as_ref(), state);
        std::ptr::hash(self.se.as_ref(), state);
    }
}
impl std::fmt::Debug for InnerNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Inner {{ nw: {:p}, ne: {:p}, sw: {:p}, se: {:p} }}",
            self.nw, self.ne, self.sw, self.se
        )
    }
}
pub struct NodeCache {
    zeros: Vec<Rc<Node>>,
    state_cache: WeakHashSet<Weak<Node>>,
    step_cache: LruCache<(Rc<Node>, NonZeroU64), Rc<Node>>,
}
impl NodeCache {
    pub fn new() -> Self {
        let mut this = Self {
            zeros: Vec::with_capacity(MAX_DEPTH.into()),
            state_cache: WeakHashSet::new(),
            step_cache: LruCache::new(LRU_CACHE_SIZE),
        };

        let mut zero = this.leaf(State::dead(), State::dead(), State::dead(), State::dead());
        while this.zeros.len() < MAX_DEPTH.into() {
            this.zeros.push(zero.clone());
            zero = this.inner(zero.clone(), zero.clone(), zero.clone(), zero.clone());
        }
        this.zeros.push(zero);

        this
    }

    fn intern(&mut self, node: Node) -> Rc<Node> {
        self.state_cache.get(&node).unwrap_or_else(|| {
            let node = Rc::new(node);
            self.state_cache.insert(node.clone());
            node
        })
    }
    fn leaf(&mut self, nw: State, ne: State, sw: State, se: State) -> Rc<Node> {
        self.intern(Node::Leaf(LeafNode::new(nw, ne, sw, se)))
    }
    fn inner(&mut self, nw: Rc<Node>, ne: Rc<Node>, sw: Rc<Node>, se: Rc<Node>) -> Rc<Node> {
        let child_depth = nw.depth();
        assert_eq!(ne.depth(), child_depth);
        assert_eq!(sw.depth(), child_depth);
        assert_eq!(se.depth(), child_depth);
        let depth = NonZeroU8::new(nw.depth() + 1).unwrap();
        self.intern(Node::Inner(depth, InnerNode::new(nw, ne, sw, se)))
    }
    fn zero(&self, depth: u8) -> &Rc<Node> {
        &self.zeros[usize::from(depth)]
    }
    fn is_zero(&self, node: &Rc<Node>) -> bool {
        Rc::ptr_eq(node, self.zero(node.depth()))
    }

    fn step_root(&mut self, node: Rc<Node>, steps: u64) -> Rc<Node> {
        match NonZeroU64::new(steps) {
            None => node,
            Some(steps) => {
                // step will reduce the area of the node, but also the unbuffered node will grow up to the same amount
                // so we need 2 buffer nodes that are both >= min_depth
                // which we do by unbuffering given node so it is >= min_depth - 1
                let min_depth = steps_to_min_depth(steps);
                let depth = self.unbufferd_depth(&node, min_depth.saturating_sub(1)) + 2;
                let node = self.buffer(node, depth);
                self.step_positive(node, steps)
            }
        }
    }

    fn step(&mut self, node: Rc<Node>, steps: u64) -> Rc<Node> {
        match NonZeroU64::new(steps) {
            Some(steps) => self.step_positive(node, steps),
            None => self.center(&node),
        }
    }

    fn step_positive(&mut self, node: Rc<Node>, steps: NonZeroU64) -> Rc<Node> {
        // annoying to have to node.clone() here
        // but .get just takes Borrow and not any other type of equivalence
        // in particular this isn't necessary if it isn't wrapped up in a tuple with steps
        self.step_cache
            .get(&(node.clone(), steps))
            .cloned()
            .unwrap_or_else(|| {
                let result = self.step_impl(&node, steps);
                self.step_cache.push((node, steps), result.clone());
                result
            })
    }

    fn step_impl(&mut self, node: &Rc<Node>, steps: NonZeroU64) -> Rc<Node> {
        let max_steps = node.max_steps();
        assert!(steps.get() <= max_steps);
        let node = node.inner().unwrap();
        // max_steps can't be 0, just assuming <= 1 will optimize better than == 1
        if max_steps <= 1 {
            let nw = node.nw.leaf().unwrap();
            let ne = node.ne.leaf().unwrap();
            let sw = node.sw.leaf().unwrap();
            let se = node.se.leaf().unwrap();

            let nw2 = State::step(
                nw.nw, nw.ne, ne.nw, nw.sw, nw.se, ne.sw, sw.nw, sw.ne, se.nw,
            );
            let ne2 = State::step(
                nw.ne, ne.nw, ne.ne, nw.se, ne.sw, ne.se, sw.ne, se.nw, se.ne,
            );
            let sw2 = State::step(
                nw.sw, nw.se, ne.sw, sw.ne, sw.ne, se.nw, sw.sw, sw.se, se.sw,
            );
            let se2 = State::step(
                nw.se, ne.sw, ne.se, sw.ne, se.nw, se.ne, sw.se, se.sw, se.se,
            );

            self.leaf(nw2, ne2, sw2, se2)
        } else {
            let nw = node.nw.inner().unwrap();
            let ne = node.ne.inner().unwrap();
            let sw = node.sw.inner().unwrap();
            let se = node.se.inner().unwrap();

            let half_steps = steps.get().saturating_sub(max_steps / 2);
            let nw2 = self.step_4(&nw.nw, &nw.ne, &nw.sw, &nw.se, half_steps);
            let nc2 = self.step_4(&nw.ne, &ne.nw, &nw.se, &ne.sw, half_steps);
            let ne2 = self.step_4(&ne.nw, &ne.ne, &ne.sw, &ne.se, half_steps);
            let cw2 = self.step_4(&nw.sw, &nw.se, &sw.nw, &sw.ne, half_steps);
            let cc2 = self.step_4(&nw.se, &ne.sw, &sw.ne, &se.nw, half_steps);
            let ce2 = self.step_4(&ne.sw, &ne.se, &se.nw, &se.ne, half_steps);
            let sw2 = self.step_4(&sw.nw, &sw.ne, &sw.sw, &sw.se, half_steps);
            let sc2 = self.step_4(&sw.ne, &se.nw, &sw.se, &se.sw, half_steps);
            let se2 = self.step_4(&se.nw, &se.ne, &se.sw, &se.se, half_steps);

            // this is always non zero as steps > 0 and half_steps < steps (since max_steps >= 2)
            let half_steps = NonZeroU64::new(steps.get() - half_steps).unwrap();
            let nw3 = self.step_4_positive(nw2, nc2.clone(), cw2.clone(), cc2.clone(), half_steps);
            let ne3 = self.step_4_positive(nc2, ne2, cc2.clone(), ce2.clone(), half_steps);
            let sw3 = self.step_4_positive(cw2, cc2.clone(), sw2, sc2.clone(), half_steps);
            let se3 = self.step_4_positive(cc2, ce2, sc2, se2, half_steps);

            self.inner(nw3, ne3, sw3, se3)
        }
    }

    fn step_4(
        &mut self,
        nw: &Rc<Node>,
        ne: &Rc<Node>,
        sw: &Rc<Node>,
        se: &Rc<Node>,
        steps: u64,
    ) -> Rc<Node> {
        match NonZeroU64::new(steps) {
            Some(steps) => {
                self.step_4_positive(nw.clone(), ne.clone(), sw.clone(), se.clone(), steps)
            }
            None => self.center_4(nw, ne, sw, se),
        }
    }

    fn step_4_positive(
        &mut self,
        nw: Rc<Node>,
        ne: Rc<Node>,
        sw: Rc<Node>,
        se: Rc<Node>,
        steps: NonZeroU64,
    ) -> Rc<Node> {
        let node = self.inner(nw, ne, sw, se);
        self.step_positive(node, steps)
    }

    fn center(&mut self, node: &Rc<Node>) -> Rc<Node> {
        let node = node.inner().unwrap();
        self.center_4(&node.nw, &node.ne, &node.sw, &node.se)
    }

    fn center_4(&mut self, nw: &Rc<Node>, ne: &Rc<Node>, sw: &Rc<Node>, se: &Rc<Node>) -> Rc<Node> {
        match (nw.as_ref(), ne.as_ref(), sw.as_ref(), se.as_ref()) {
            (Node::Leaf(nw), Node::Leaf(ne), Node::Leaf(sw), Node::Leaf(se)) => {
                self.leaf(nw.se, ne.sw, sw.ne, se.nw)
            }
            (Node::Inner(_, nw), Node::Inner(_, ne), Node::Inner(_, sw), Node::Inner(_, se)) => {
                self.inner(nw.se.clone(), ne.sw.clone(), sw.ne.clone(), se.nw.clone())
            }
            _ => panic!("inconsistent node depth"),
        }
    }

    fn buffer(&mut self, node: Rc<Node>, to_depth: u8) -> Rc<Node> {
        let node_depth = node.depth();
        match node_depth.cmp(&to_depth) {
            std::cmp::Ordering::Equal => node,
            std::cmp::Ordering::Greater => {
                let InnerNode { nw, ne, sw, se } = &node.inner().unwrap();
                self.shrink_4(nw, ne, sw, se, to_depth)
            }
            std::cmp::Ordering::Less => {
                let InnerNode { nw, ne, sw, se } = node.inner().unwrap().clone();
                self.expand_4(nw, ne, sw, se, to_depth)
            }
        }
    }
    fn shrink_4(
        &mut self,
        nw: &Rc<Node>,
        ne: &Rc<Node>,
        sw: &Rc<Node>,
        se: &Rc<Node>,
        to_depth: u8,
    ) -> Rc<Node> {
        if nw.depth() == to_depth {
            self.center_4(nw, ne, sw, se)
        } else {
            let nw = &nw.inner().unwrap().se;
            let ne = &ne.inner().unwrap().sw;
            let sw = &sw.inner().unwrap().ne;
            let se = &se.inner().unwrap().nw;
            self.shrink_4(nw, ne, sw, se, to_depth)
        }
    }
    fn expand_4(
        &mut self,
        nw: Rc<Node>,
        ne: Rc<Node>,
        sw: Rc<Node>,
        se: Rc<Node>,
        to_depth: u8,
    ) -> Rc<Node> {
        if nw.depth() + 1 == to_depth {
            self.inner(nw, ne, sw, se)
        } else {
            let zero = self.zero(nw.depth()).clone();
            let nw = self.inner(zero.clone(), zero.clone(), zero.clone(), nw);
            let ne = self.inner(zero.clone(), zero.clone(), ne, zero.clone());
            let sw = self.inner(zero.clone(), sw, zero.clone(), zero.clone());
            let se = self.inner(se, zero.clone(), zero.clone(), zero);
            self.expand_4(nw, ne, sw, se, to_depth)
        }
    }

    // find the smallest depth where the node is unbuffered, maxed with target_depth
    fn unbufferd_depth(&self, node: &Rc<Node>, target_depth: u8) -> u8 {
        let node_depth = node.depth();
        if node_depth <= target_depth {
            return target_depth;
        }

        let InnerNode { nw, ne, sw, se } = &node.inner().unwrap();
        self.unbufferd_depth_4(nw, ne, sw, se, target_depth)
    }
    fn unbufferd_depth_4(
        &self,
        nw: &Rc<Node>,
        ne: &Rc<Node>,
        sw: &Rc<Node>,
        se: &Rc<Node>,
        target_depth: u8,
    ) -> u8 {
        let child_depth = nw.depth();
        if child_depth < target_depth {
            target_depth
        } else if self.is_buffered(nw, ne, sw, se) {
            match (nw.as_ref(), ne.as_ref(), sw.as_ref(), se.as_ref()) {
                (
                    Node::Inner(_, nw),
                    Node::Inner(_, ne),
                    Node::Inner(_, sw),
                    Node::Inner(_, se),
                ) => self.unbufferd_depth_4(&nw.se, &ne.sw, &sw.ne, &se.nw, target_depth),
                _ => child_depth,
            }
        } else {
            child_depth
        }
    }

    fn is_buffered(&self, nw: &Rc<Node>, ne: &Rc<Node>, sw: &Rc<Node>, se: &Rc<Node>) -> bool {
        match (nw.as_ref(), ne.as_ref(), sw.as_ref(), se.as_ref()) {
            (Node::Leaf(nw), Node::Leaf(ne), Node::Leaf(sw), Node::Leaf(se)) => {
                nw.nw.is_dead()
                    && nw.ne.is_dead()
                    && nw.sw.is_dead()
                    && ne.nw.is_dead()
                    && ne.ne.is_dead()
                    && ne.se.is_dead()
                    && sw.nw.is_dead()
                    && sw.sw.is_dead()
                    && sw.se.is_dead()
                    && se.ne.is_dead()
                    && se.sw.is_dead()
                    && se.se.is_dead()
            }
            (
                Node::Inner(depth, nw),
                Node::Inner(_, ne),
                Node::Inner(_, sw),
                Node::Inner(_, se),
            ) => {
                let zero = self.zero(depth.get() - 1);
                Rc::ptr_eq(&nw.nw, zero)
                    && Rc::ptr_eq(&nw.ne, zero)
                    && Rc::ptr_eq(&nw.sw, zero)
                    && Rc::ptr_eq(&ne.nw, zero)
                    && Rc::ptr_eq(&ne.ne, zero)
                    && Rc::ptr_eq(&ne.se, zero)
                    && Rc::ptr_eq(&sw.nw, zero)
                    && Rc::ptr_eq(&sw.sw, zero)
                    && Rc::ptr_eq(&sw.se, zero)
                    && Rc::ptr_eq(&se.ne, zero)
                    && Rc::ptr_eq(&se.sw, zero)
                    && Rc::ptr_eq(&se.se, zero)
            }
            _ => panic!("inconsistent node depth"),
        }
    }
}
