use std::collections::HashMap;
use std::hash::Hash;

pub type Pos = glam::IVec2;

// it should be possible to get an extra bit of space by using u32 and being careful about converting between i32 and u32
const MAX_HALF_WIDTH: i32 = 1 << 30;
const MAX_POS: i32 = MAX_HALF_WIDTH - 1;
const MIN_POS: i32 = -MAX_HALF_WIDTH;

#[derive(Clone, Copy, Debug)]
struct Neighbors<T> {
    n: T,
    ne: T,
    e: T,
    se: T,
    s: T,
    sw: T,
    w: T,
    nw: T,
}
impl<T> Neighbors<T> {
    fn map<U>(self, map: impl FnMut(T) -> U) -> Neighbors<U> {
        Neighbors {
            n: map(self.n),
            ne: map(self.n),
            e: map(self.n),
            se: map(self.n),
            s: map(self.n),
            sw: map(self.n),
            w: map(self.n),
            nw: map(self.n),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct State(u32);
impl State {
    pub fn new(raw: u32) -> Self {
        Self(raw)
    }
    pub fn zero() -> Self {
        Self(0)
    }
    pub fn update(self, neighbors: Neighbors<State>) -> State {
        // requireing that 0 maps to 0 could be relaxed
        // but it simplifies things for now
        if self.0 == 0 {
            return self;
        }

        todo!("implement")
    }
}
impl Default for State {
    fn default() -> Self {
        Self::zero()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Index(u32);
impl Index {
    fn zero() -> Self {
        Self(0)
    }
}
impl Default for Index {
    fn default() -> Self {
        Self::zero()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct IndexOrState(u32);
impl IndexOrState {
    fn zero() -> Self {
        Self(0)
    }
    fn as_index(self) -> Index {
        Index(self.0)
    }
    fn as_state(self) -> State {
        State(self.0)
    }
}
impl Default for IndexOrState {
    fn default() -> Self {
        Self::zero()
    }
}
impl From<Index> for IndexOrState {
    fn from(index: Index) -> Self {
        Self(index.0)
    }
}
impl From<State> for IndexOrState {
    fn from(state: State) -> Self {
        Self(state.0)
    }
}

#[cfg(not(debug_assertions))]
mod node_indexer {
    use super::*;

    #[derive(Debug)]
    pub struct NodeIndexer {
        to_node: Vec<RawNode>,
        to_index: HashMap<RawNode, Index>,
    }
    impl NodeIndexer {
        pub fn new() -> Self {
            let to_node = vec![RawNode::empty()];
            let mut to_index = HashMap::new();
            to_index.insert(RawNode::empty(), Index::zero());
            Self { to_node, to_index }
        }
        pub fn get_node(&self, index: Index, half_width: i32) -> RawNode {
            self.to_node[index.0 as usize]
        }
        pub fn get_index(&mut self, node: RawNode, half_width: i32) -> Index {
            *self.to_index.entry(node).or_insert_with(|| {
                let next_index = Index(self.to_node.len().try_into().expect("too many nodes"));
                self.to_node.push(node);
                next_index
            })
        }
    }
    impl Default for NodeIndexer {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(debug_assertions)]
mod node_indexer {
    use super::*;

    #[derive(Debug)]
    pub struct NodeIndexer {
        to_node: Vec<(RawNode, i32)>,
        to_index: HashMap<RawNode, (Index, i32)>,
    }
    impl NodeIndexer {
        pub fn new() -> Self {
            let to_node = vec![(RawNode::empty(), 0)];
            let mut to_index = HashMap::new();
            to_index.insert(RawNode::empty(), (Index::zero(), 0));
            Self { to_node, to_index }
        }
        pub fn get_node(&self, index: Index, half_width: i32) -> RawNode {
            assert!(half_width > 0 && (half_width as u32).is_power_of_two());

            let (node, node_half_width) = self.to_node[index.0 as usize];

            if node_half_width != 0 {
                assert_eq!(node_half_width, half_width);
            }
            node
        }
        pub fn get_index(&mut self, node: RawNode, half_width: i32) -> Index {
            assert!(half_width > 0 && (half_width as u32).is_power_of_two());

            let (index, index_half_width) = *self.to_index.entry(node).or_insert_with(|| {
                let next_index = Index(self.to_node.len().try_into().expect("too many nodes"));
                self.to_node.push((node, half_width));
                (next_index, half_width)
            });

            if index_half_width != 0 {
                assert_eq!(index_half_width, half_width);
            }
            index
        }
    }
    impl Default for NodeIndexer {
        fn default() -> Self {
            Self::new()
        }
    }
}
use node_indexer::NodeIndexer;

pub struct Board {
    indexer: NodeIndexer,
    // is a power of 2
    half_width: i32,
    root: Index,
    // 0,0 is the north west most cell of the south east child of the root node
    // x increases to the east
    // y increases to the south
}
impl Board {
    pub fn new() -> Self {
        Self {
            indexer: NodeIndexer::new(),
            half_width: 1,
            root: Index::zero(),
        }
    }
    pub fn get(&self, pos: Pos) -> State {
        if pos.x >= self.half_width
            || pos.x < -self.half_width
            || pos.y >= self.half_width
            || pos.y < -self.half_width
        {
            State::zero()
        } else {
            self.get_in(self.half_width, self.root, pos)
        }
    }
    fn get_in(&self, half_width: i32, index: Index, pos: Pos) -> State {
        if index == Index::zero() {
            State::zero()
        } else {
            let node = self.indexer.get_node(index, half_width);
            if half_width == 1 {
                match pos.as_ref() {
                    [0, 0] => node.se.as_state(),
                    [-1, 0] => node.ne.as_state(),
                    [0, -1] => node.sw.as_state(),
                    [-1, -1] => node.nw.as_state(),
                    _ => panic!("pos out of bounds"),
                }
            } else {
                let quarter = half_width / 2;
                if pos.x >= 0 {
                    if pos.y >= 0 {
                        self.get_in(
                            quarter,
                            node.se.as_index(),
                            pos + Pos::new(-quarter, -quarter),
                        )
                    } else {
                        self.get_in(
                            quarter,
                            node.ne.as_index(),
                            pos + Pos::new(-quarter, quarter),
                        )
                    }
                } else {
                    if pos.y >= 0 {
                        self.get_in(
                            quarter,
                            node.sw.as_index(),
                            pos + Pos::new(quarter, -quarter),
                        )
                    } else {
                        self.get_in(
                            quarter,
                            node.nw.as_index(),
                            pos + Pos::new(quarter, quarter),
                        )
                    }
                }
            }
        }
    }

    pub fn set(&mut self, pos: Pos, state: State) {
        self.expand_to(pos);
        self.root = self.set_in(self.half_width, self.root, pos, state)
    }
    fn expand_to(&mut self, pos: Pos) {
        let max_x = if pos.x >= 0 { pos.x } else { -1 - pos.x };
        let max_y = if pos.y >= 0 { pos.y } else { -1 - pos.y };
        let max_pos = max_x.max(max_y);
        if max_pos < self.half_width {
            return;
        }
        if max_pos >= MAX_HALF_WIDTH {
            panic!("pos {} too big", pos)
        }

        let empty = RawNode::empty();
        let mut node = self.indexer.get_node(self.root, self.half_width);

        while max_pos >= self.half_width {
            let nw = self
                .indexer
                .get_index(empty.with_se(node.nw), self.half_width);
            let ne = self
                .indexer
                .get_index(empty.with_sw(node.ne), self.half_width);
            let sw = self
                .indexer
                .get_index(empty.with_ne(node.sw), self.half_width);
            let se = self
                .indexer
                .get_index(empty.with_nw(node.se), self.half_width);
            self.half_width *= 2;
            node = RawNode::new_inner(nw, ne, sw, se);
        }
        self.root = self.indexer.get_index(node, self.half_width);
    }
    fn set_in(&mut self, half_width: i32, index: Index, pos: Pos, state: State) -> Index {
        let node = self.indexer.get_node(index, half_width);
        let new_node = if half_width == 1 {
            match pos.as_ref() {
                [0, 0] => node.with_se(state.into()),
                [-1, 0] => node.with_ne(state.into()),
                [0, -1] => node.with_sw(state.into()),
                [-1, -1] => node.with_nw(state.into()),
                _ => panic!("pos out of bounds"),
            }
        } else {
            let quarter = half_width / 2;
            if pos.x >= 0 {
                if pos.y >= 0 {
                    node.with_se(
                        self.set_in(
                            quarter,
                            node.se.as_index(),
                            pos + Pos::new(-quarter, -quarter),
                            state,
                        )
                        .into(),
                    )
                } else {
                    node.with_ne(
                        self.set_in(
                            quarter,
                            node.ne.as_index(),
                            pos + Pos::new(-quarter, quarter),
                            state,
                        )
                        .into(),
                    )
                }
            } else {
                if pos.y >= 0 {
                    node.with_sw(
                        self.set_in(
                            quarter,
                            node.sw.as_index(),
                            pos + Pos::new(quarter, -quarter),
                            state,
                        )
                        .into(),
                    )
                } else {
                    node.with_nw(
                        self.set_in(
                            quarter,
                            node.nw.as_index(),
                            pos + Pos::new(quarter, quarter),
                            state,
                        )
                        .into(),
                    )
                }
            }
        };
        self.indexer.get_index(new_node, half_width)
        // refcount dec_inex(index)
    }

    pub fn update(&mut self, steps: u32) {
        if steps == 0 {
            return;
        }
        assert!(steps.is_power_of_two(), "steps must be a power of two");

        // TODO handle steps > self.half_width
    }

    fn update_in(
        &mut self,
        half_width: i32,
        steps: u32,
        index: IndexOrState,
        neighbors: Neighbors<IndexOrState>,
    ) -> IndexOrState {
        // based on assumptions in state update
        if index == IndexOrState::zero() {
            return index;
        }
        // half_width is positive, so as behaves sanely
        assert!(half_width as u32 >= steps);

        if half_width as u32 == steps {
            if steps == 1 {
                index
                    .as_state()
                    .update(neighbors.map(IndexOrState::as_state))
                    .into()
                    // refcount dec_inex(index)
            } else {
                // TODO cache
                self.update_node(
                    half_width,
                    steps,
                    index.as_index(),
                    neighbors.map(IndexOrState::as_index),
                )
                .into()
            }
        } else {
            self.update_large_node(
                half_width,
                steps,
                index.as_index(),
                neighbors.map(IndexOrState::as_index),
            )
            .into()
        }
    }

    fn update_large_node(
        &mut self,
        half_width: i32,
        steps: u32,
        index: Index,
        neighbors: Neighbors<Index>,
    ) -> Index {
        assert!(half_width as u32 > steps);
        assert!(steps >= 1);
        let node = self.indexer.get_node(index, half_width);
        let neighbors = neighbors.map(|index| self.indexer.get_node(index, half_width));

        let node = self.update_4_in(
            half_width,
            steps,
            RawNode::new(neighbors.nw.se, neighbors.n.sw, neighbors.w.ne, node.nw),
            todo!("ne"),
            todo!("sw"),
            todo!("se"),
        );

        self.indexer.get_index(node, half_width).into()
        // refcount dec_inex(index)
    }

    fn update_node(
        &mut self,
        half_width: i32,
        steps: u32,
        index: Index,
        neighbors: Neighbors<Index>,
    ) -> Index {
        assert!(half_width as u32 == steps);
        assert!(steps >= 2);
        let half_steps = steps / 2;
        let node = self.indexer.get_node(index, half_width);
        let neighbors = neighbors.map(|index| self.indexer.get_node(index, half_width));
        let nw = self.update_4_in(
            half_width,
            half_steps,
            neighbors.nw,
            neighbors.n,
            neighbors.w,
            node,
        );
        let ne = self.update_4_in(
            half_width,
            half_steps,
            neighbors.n,
            neighbors.ne,
            node,
            neighbors.e,
        );
        let sw = todo!();
        let se = todo!();
        let node = self.update_4_in(half_width, half_steps, nw, ne, sw, se);
        self.indexer.get_index(node, half_width).into()
        // refcount dec_inex(index)
    }

    fn update_4_in(
        &mut self,
        half_width: i32,
        steps: u32,
        nw: RawNode,
        ne: RawNode,
        sw: RawNode,
        se: RawNode,
    ) -> RawNode {
        let quarter_width = half_width / 2;
        let nw_index = self.update_in(
            quarter_width,
            steps,
            nw.se,
            Neighbors {
                n: nw.ne,
                ne: ne.nw,
                e: ne.sw,
                se: se.nw,
                s: sw.ne,
                sw: sw.nw,
                w: nw.sw,
                nw: nw.nw,
            },
        );
        let ne_index = self.update_in(
            quarter_width,
            steps,
            ne.sw,
            Neighbors {
                n: ne.nw,
                ne: ne.ne,
                e: ne.se,
                se: se.ne,
                s: se.nw,
                sw: sw.ne,
                w: nw.se,
                nw: nw.ne,
            },
        );
        let sw_index = todo!();
        let se_index = todo!();
        RawNode::new(nw_index, ne_index, sw_index, se_index)
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct RawNode {
    nw: IndexOrState,
    ne: IndexOrState,
    sw: IndexOrState,
    se: IndexOrState,
}
impl RawNode {
    fn empty() -> Self {
        Self::new(
            IndexOrState::zero(),
            IndexOrState::zero(),
            IndexOrState::zero(),
            IndexOrState::zero(),
        )
    }
    fn new(nw: IndexOrState, ne: IndexOrState, sw: IndexOrState, se: IndexOrState) -> Self {
        Self { nw, ne, sw, se }
    }
    fn new_inner(nw: Index, ne: Index, sw: Index, se: Index) -> Self {
        Self::new(nw.into(), ne.into(), sw.into(), se.into())
    }

    fn with_nw(self, nw: IndexOrState) -> Self {
        Self { nw, ..self }
    }
    fn with_ne(self, ne: IndexOrState) -> Self {
        Self { ne, ..self }
    }
    fn with_sw(self, sw: IndexOrState) -> Self {
        Self { sw, ..self }
    }
    fn with_se(self, se: IndexOrState) -> Self {
        Self { se, ..self }
    }
}

#[cfg(test)]
mod test {
    use super::{Board, Pos, State};

    #[test]
    fn basic() {
        let mut b = Board::new();
        let pos = Pos::new(2, -5);
        let state = State::new(5);
        b.set(pos, state);
        assert_eq!(b.get(pos), state);
    }
}
