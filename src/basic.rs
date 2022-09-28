use std::hash::Hash;
use std::num::{NonZeroU64, NonZeroU8, NonZeroUsize};
use std::rc::{Rc, Weak};

use lru::LruCache;
use weak_table::WeakHashSet;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct State(u32);
impl State {
    fn zero() -> Self {
        Self(0)
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
        todo!()
    }
}

const MAX_DEPTH: u8 = 63;
const MAX_STEPS: u64 = 1 << 62;

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
fn steps_to_min_depth(steps: u64) -> u8 {
    assert!(
        steps <= MAX_STEPS,
        "steps {} must be <= {}",
        steps,
        MAX_STEPS
    );
    match NonZeroU64::new(steps) {
        None => 0,
        Some(steps) => {
            if steps.is_power_of_two() {
                steps.ilog2() as u8 + 1
            } else {
                steps.ilog2() as u8 + 2
            }
        }
    }
}
// the absolute limit on width is 2^64, which is not representable
// where as half_width is representable and is often more useful
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

#[derive(Clone, PartialEq, Eq, Hash)]
enum Node {
    Leaf(LeafNode),
    Inner(InnerNode),
}
impl Node {
    fn depth(&self) -> u8 {
        match self {
            Self::Leaf(_) => 0,
            Self::Inner(inner) => inner.depth.get(),
        }
    }
    fn max_steps(&self) -> u64 {
        depth_to_max_steps(self.depth())
    }
    fn half_width(&self) -> u64 {
        depth_to_half_width(self.depth())
    }

    fn leaf(&self) -> Result<&LeafNode, &InnerNode> {
        match self {
            Self::Leaf(leaf) => Ok(leaf),
            Self::Inner(inner) => Err(inner),
        }
    }
    fn inner(&self) -> Result<&InnerNode, &LeafNode> {
        match self {
            Self::Leaf(leaf) => Err(leaf),
            Self::Inner(inner) => Ok(inner),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct LeafNode {
    // these could be made into Rc<StateBlock> or something
    // to allow for efficient leaf representations
    nw: State,
    ne: State,
    sw: State,
    se: State,
}
impl LeafNode {
    fn new(nw: State, ne: State, sw: State, se: State) -> Self {
        Self { nw, ne, sw, se }
    }

    fn center(nw: &Self, ne: &Self, sw: &Self, se: &Self) -> Self {
        Self::new(nw.se, ne.sw, sw.ne, se.nw)
    }

    fn step_4(nw: &Self, ne: &Self, sw: &Self, se: &Self) -> Self {
        Self::new(
            State::step(
                nw.nw, nw.ne, ne.nw, nw.sw, nw.se, ne.sw, sw.nw, sw.ne, se.nw,
            ),
            State::step(
                nw.ne, ne.nw, ne.ne, nw.se, ne.sw, ne.se, sw.ne, se.nw, se.ne,
            ),
            State::step(
                nw.sw, nw.se, ne.sw, sw.ne, sw.ne, se.nw, sw.sw, sw.se, se.sw,
            ),
            State::step(
                nw.se, ne.sw, ne.se, sw.ne, se.nw, se.ne, sw.se, se.sw, se.se,
            ),
        )
    }
}

#[derive(Clone, Eq)]
struct InnerNode {
    depth: NonZeroU8,
    nw: Rc<Node>,
    ne: Rc<Node>,
    sw: Rc<Node>,
    se: Rc<Node>,
}
impl InnerNode {
    fn new(nw: Rc<Node>, ne: Rc<Node>, sw: Rc<Node>, se: Rc<Node>) -> Self {
        let child_depth = nw.depth();
        assert_eq!(ne.depth(), child_depth);
        assert_eq!(sw.depth(), child_depth);
        assert_eq!(se.depth(), child_depth);
        let depth = child_depth + 1;
        assert!(depth <= 63, "depth too large");
        Self {
            depth: NonZeroU8::new(depth).unwrap(),
            nw,
            ne,
            sw,
            se,
        }
    }

    fn center(nw: &Self, ne: &Self, sw: &Self, se: &Self) -> Self {
        Self::new(nw.se.clone(), ne.sw.clone(), sw.ne.clone(), se.nw.clone())
    }

    fn max_steps(&self) -> u64 {
        depth_to_max_steps(self.depth.get())
    }
    fn half_width(&self) -> u64 {
        depth_to_half_width(self.depth.get())
    }
}
impl PartialEq for InnerNode {
    fn eq(&self, other: &Self) -> bool {
        // probably possible to ignore depth, but not doing so for now
        self.depth == other.depth
            && Rc::ptr_eq(&self.nw, &other.nw)
            && Rc::ptr_eq(&self.ne, &other.ne)
            && Rc::ptr_eq(&self.sw, &other.sw)
            && Rc::ptr_eq(&self.se, &other.se)
    }
}
impl std::hash::Hash for InnerNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.depth.hash(state);
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
            "Inner {{ depth: {:?}, nw: {:p}, ne: {:p}, sw: {:p}, se: {:p} }}",
            self.depth, self.nw, self.ne, self.sw, self.se
        )
    }
}
struct NodeCache {
    state_cache: WeakHashSet<Weak<Node>>,
    step_cache: LruCache<(Rc<Node>, NonZeroU64), Rc<Node>>,
}
impl NodeCache {
    fn new(cap: NonZeroUsize) -> Self {
        Self {
            state_cache: WeakHashSet::new(),
            step_cache: LruCache::new(cap),
        }
    }
    fn set_cap(&mut self, cap: NonZeroUsize) {
        self.step_cache.resize(cap)
    }

    fn intern(&mut self, node: Node) -> Rc<Node> {
        self.state_cache.get(&node).unwrap_or_else(|| {
            let node = Rc::new(node);
            self.state_cache.insert(node.clone());
            node
        })
    }
    fn zero(&mut self, depth: u8) -> Rc<Node> {
        if depth == 0 {
            self.leaf(State::zero(), State::zero(), State::zero(), State::zero())
        } else {
            let sub = self.zero(depth - 1);
            self.inner(sub.clone(), sub.clone(), sub.clone(), sub)
        }
    }
    fn leaf(&mut self, nw: State, ne: State, sw: State, se: State) -> Rc<Node> {
        self.intern(Node::Leaf(LeafNode::new(nw, ne, sw, se)))
    }
    fn inner(&mut self, nw: Rc<Node>, ne: Rc<Node>, sw: Rc<Node>, se: Rc<Node>) -> Rc<Node> {
        self.intern(Node::Inner(InnerNode::new(nw, ne, sw, se)))
    }

    fn step_root(&mut self, node: Rc<Node>, steps: u64) -> Rc<Node> {
        match NonZeroU64::new(steps) {
            None => node,
            Some(steps) => {
                let z = self.zero(node.depth());
                // this is the right idea, but may need to expand more than once
                // start by doing self.center_4(...) for each
                // then nw = self.inner(z, z, z, nw) for each (noting that z must grow each iteration)
                let nw = self.step_4_positive(z.clone(), z.clone(), z.clone(), node.clone(), steps);
                let ne = self.step_4_positive(z.clone(), z.clone(), node.clone(), z.clone(), steps);
                let sw = self.step_4_positive(z.clone(), node.clone(), z.clone(), z.clone(), steps);
                let se = self.step_4_positive(node, z.clone(), z.clone(), z, steps);
                self.inner(nw, ne, sw, se)
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
        // but .get just takes borrow and not any other type of equivalence
        // in particularit isn't necessary if it isn't wrapped up in a tuple with steps
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
        let node = node.inner().expect("can't step a leaf");
        let max_steps = node.max_steps();
        assert!(steps.get() <= max_steps);
        // max_steps can't be 0, just assuming <= 1 will optimize better than == 1
        if max_steps <= 1 {
            let nw = node.nw.leaf().unwrap();
            let ne = node.ne.leaf().unwrap();
            let sw = node.sw.leaf().unwrap();
            let se = node.se.leaf().unwrap();
            self.intern(Node::Leaf(LeafNode::step_4(nw, ne, sw, se)))
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
        let node = node.inner().expect("can't get the center of a leaf");
        self.center_4(&node.nw, &node.ne, &node.sw, &node.se)
    }

    fn center_4(&mut self, nw: &Rc<Node>, ne: &Rc<Node>, sw: &Rc<Node>, se: &Rc<Node>) -> Rc<Node> {
        match (nw.as_ref(), ne.as_ref(), sw.as_ref(), se.as_ref()) {
            (Node::Leaf(nw), Node::Leaf(ne), Node::Leaf(sw), Node::Leaf(se)) => {
                self.intern(Node::Leaf(LeafNode::center(nw, ne, sw, se)))
            }
            (Node::Inner(nw), Node::Inner(ne), Node::Inner(sw), Node::Inner(se)) => {
                self.intern(Node::Inner(InnerNode::center(nw, ne, sw, se)))
            }
            _ => panic!("inconsistent node depth"),
        }
    }
}
