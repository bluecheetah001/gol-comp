use crate::{DepthQuad, Node, Population, Pos, Quad};

impl Node {
    /// returned Node's 0,0 is at returned Pos in self's coordinate space
    pub fn offset_norm(&self) -> (Pos, Self) {
        self.inner()
            .and_then(Quad::offset_norm)
            .unwrap_or_else(|| (Pos::new(0, 0), self.clone()))
    }
    fn offset_norm_plus(&self, pos: Pos) -> (Pos, Self) {
        let (pos2, node) = self.offset_norm();
        (pos + pos2, node)
    }

    /// layed out in reading order just like Block
    /// padded to 16 bits so that this can be shifted and unioned with 3 other nodes
    /// high bits
    ///  0  0  0  0
    ///  0  0  0  0
    ///  0  0 nw ne
    ///  0  0 sw se
    /// low bits
    fn filled_flags(&self) -> u16 {
        match self.depth_quad() {
            DepthQuad::Leaf(leaf) => {
                let nw = if leaf.nw.is_empty() { 0 } else { 32 };
                let ne = if leaf.ne.is_empty() { 0 } else { 16 };
                let sw = if leaf.ne.is_empty() { 0 } else { 2 };
                let se = if leaf.ne.is_empty() { 0 } else { 1 };
                nw | ne | sw | se
            }
            DepthQuad::Inner(_, inner) => {
                let nw = if inner.nw.is_empty() { 0 } else { 32 };
                let ne = if inner.ne.is_empty() { 0 } else { 16 };
                let sw = if inner.ne.is_empty() { 0 } else { 2 };
                let se = if inner.ne.is_empty() { 0 } else { 1 };
                nw | ne | sw | se
            }
        }
    }
}

impl Quad<Node> {
    fn offset_norm(&self) -> Option<(Pos, Node)> {
        let filled_flags = (self.nw.filled_flags() << 6)
            | (self.ne.filled_flags() << 4)
            | (self.sw.filled_flags() << 2)
            | self.se.filled_flags();
        // if one is None and the other is Max or Min, then could center it even though it wouldn't be smaller
        let case_v = JoinCase::from_filled_v(filled_flags)?;
        let case_h = JoinCase::from_filled_h(filled_flags)?;

        let w = case_v.join_v(&self.nw, &self.sw);
        let e = case_v.join_v(&self.ne, &self.se);
        let node = case_h.join_h(&w, &e);
        let pos = Pos::new(
            case_h.offset(node.half_width()),
            case_v.offset(node.half_width()),
        );
        Some(node.offset_norm_plus(pos))
    }
}

#[derive(Debug, Clone, Copy)]
enum JoinCase {
    Min,
    Mid,
    Max,
}
impl JoinCase {
    fn from_filled_v(flags: u16) -> Option<Self> {
        let flags = flags | (flags >> 1);
        let flags = flags | (flags >> 2);
        match flags & 0x1111 {
            0x0000 | 0x0010 | 0x0100 | 0x0110 => Some(JoinCase::Mid),
            0x1100 | 0x1000 => Some(JoinCase::Min),
            0x0011 | 0x0001 => Some(JoinCase::Max),
            _ => None,
        }
    }
    fn from_filled_h(flags: u16) -> Option<Self> {
        let flags = flags | (flags >> 4);
        let flags = flags | (flags >> 8);
        match flags & 0b1111 {
            0b0000 | 0b0010 | 0b0100 | 0b0110 => Some(JoinCase::Mid),
            0b1100 | 0b1000 => Some(JoinCase::Min),
            0b0011 | 0b0001 => Some(JoinCase::Max),
            _ => None,
        }
    }
    fn offset(self, amount: i64) -> i64 {
        match self {
            JoinCase::Min => -amount,
            JoinCase::Mid => 0,
            JoinCase::Max => amount,
        }
    }
    fn join_v(self, n: &Node, s: &Node) -> Node {
        match self {
            JoinCase::Min => n.clone(),
            JoinCase::Max => s.clone(),
            JoinCase::Mid => match (n.depth_quad(), s.depth_quad()) {
                (DepthQuad::Leaf(n), DepthQuad::Leaf(s)) => Node::new_leaf(Quad {
                    nw: n.sw,
                    ne: n.se,
                    sw: s.nw,
                    se: s.ne,
                }),
                (DepthQuad::Inner(depth, n), DepthQuad::Inner(_, s)) => Node::new_depth_inner(
                    *depth,
                    Quad {
                        nw: n.sw.clone(),
                        ne: n.se.clone(),
                        sw: s.nw.clone(),
                        se: s.ne.clone(),
                    },
                ),
                _ => panic!("inconsistent depth"),
            },
        }
    }
    fn join_h(self, w: &Node, e: &Node) -> Node {
        match self {
            JoinCase::Min => w.clone(),
            JoinCase::Max => e.clone(),
            JoinCase::Mid => match (w.depth_quad(), e.depth_quad()) {
                (DepthQuad::Leaf(w), DepthQuad::Leaf(e)) => Node::new_leaf(Quad {
                    nw: w.ne,
                    ne: e.nw,
                    sw: w.se,
                    se: e.sw,
                }),
                (DepthQuad::Inner(depth, w), DepthQuad::Inner(_, e)) => Node::new_depth_inner(
                    *depth,
                    Quad {
                        nw: w.ne.clone(),
                        ne: e.nw.clone(),
                        sw: w.se.clone(),
                        se: e.sw.clone(),
                    },
                ),
                _ => panic!("inconsistent depth"),
            },
        }
    }
}

// TODO testing!
