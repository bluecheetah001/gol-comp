use crate::{DepthQuad, Node, Population, Pos, Quad};

impl Node {
    fn offset_norm(&self) -> (Pos, Self) {
        self.inner()
            .and_then(|inner| inner.as_ref().offset_norm())
            .unwrap_or_else(|| (Pos::new(0, 0), self.clone()))
    }
}

impl Quad<&Node> {
    #[allow(clippy::too_many_lines)]
    fn offset_norm(self) -> Option<(Pos, Node)> {
        #[allow(clippy::unnecessary_wraps)]
        fn recurse(pos: Pos, node: &Node) -> Option<(Pos, Node)> {
            let (pos2, node) = node.offset_norm();
            Some((pos + pos2, node))
        }
        fn recurse_v(x: i64, n: &Node, s: &Node) -> Option<(Pos, Node)> {
            match (n.depth_quad(), s.depth_quad()) {
                (DepthQuad::Leaf(n), DepthQuad::Leaf(s)) => {
                    if n.nw.is_empty() && n.ne.is_empty() && s.sw.is_empty() && s.se.is_empty() {
                        Some((
                            Pos::new(x, 0),
                            Quad {
                                nw: n.sw,
                                ne: n.se,
                                sw: s.nw,
                                se: s.ne,
                            }
                            .into(),
                        ))
                    } else {
                        None
                    }
                }
                (DepthQuad::Inner(_, n), DepthQuad::Inner(_, s)) => {
                    if n.nw.is_empty() && n.ne.is_empty() && s.sw.is_empty() && s.se.is_empty() {
                        let pos = Pos::new(x, 0);
                        let inner = Quad {
                            nw: &n.sw,
                            ne: &n.se,
                            sw: &s.nw,
                            se: &s.ne,
                        };
                        if let Some((pos2, node)) = inner.offset_norm() {
                            Some((pos + pos2, node))
                        } else {
                            Some((pos, inner.cloned().into()))
                        }
                    } else {
                        None
                    }
                }
                _ => panic!("inconsistent depth"),
            }
        }
        fn recurse_h(y: i64, w: &Node, e: &Node) -> Option<(Pos, Node)> {
            match (w.depth_quad(), e.depth_quad()) {
                (DepthQuad::Leaf(w), DepthQuad::Leaf(e)) => {
                    if w.nw.is_empty() && w.sw.is_empty() && e.ne.is_empty() && e.se.is_empty() {
                        Some((
                            Pos::new(0, y),
                            Quad {
                                nw: w.ne,
                                ne: e.nw,
                                sw: w.se,
                                se: e.sw,
                            }
                            .into(),
                        ))
                    } else {
                        None
                    }
                }
                (DepthQuad::Inner(_, w), DepthQuad::Inner(_, e)) => {
                    if w.nw.is_empty() && w.sw.is_empty() && e.ne.is_empty() && e.se.is_empty() {
                        let pos = Pos::new(0, y);
                        let inner = Quad {
                            nw: &w.ne,
                            ne: &e.nw,
                            sw: &w.se,
                            se: &e.sw,
                        };
                        if let Some((pos2, node)) = inner.offset_norm() {
                            Some((pos + pos2, node))
                        } else {
                            Some((pos, inner.cloned().into()))
                        }
                    } else {
                        None
                    }
                }
                _ => panic!("inconsistent depth"),
            }
        }

        let nw = if self.nw.is_empty() { 0 } else { 8 };
        let ne = if self.ne.is_empty() { 0 } else { 4 };
        let sw = if self.sw.is_empty() { 0 } else { 2 };
        let se = if self.se.is_empty() { 0 } else { 1 };
        #[allow(clippy::cast_possible_wrap)] // not max depth
        let half_width = self.nw.width() as i64;
        match nw | ne | sw | se {
            0 => Some((Pos::new(0, 0), Node::empty(0))),
            1 => recurse(Pos::new(half_width, half_width), self.se),
            2 => recurse(Pos::new(half_width, -half_width), self.sw),
            3 => recurse_h(half_width, self.sw, self.se),
            4 => recurse(Pos::new(-half_width, half_width), self.ne),
            5 => recurse_v(half_width, self.ne, self.se),
            8 => recurse(Pos::new(-half_width, -half_width), self.nw),
            10 => recurse_v(-half_width, self.nw, self.sw),
            12 => recurse_h(-half_width, self.nw, self.ne),
            _ => todo!("check if center buffered"),
        }
    }
}
