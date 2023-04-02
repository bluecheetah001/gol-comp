use crate::pos::Pos;
use crate::{Block, DepthQuad, Node, Quadrant};

impl Node {
    pub fn get(&self, pos: Pos) -> bool {
        let half_width = self.half_width();
        if pos.x >= half_width || pos.y >= half_width || pos.x < -half_width || pos.y < -half_width
        {
            false
        } else {
            self.get_in_bounds(pos)
        }
    }
    fn get_in_bounds(&self, pos: Pos) -> bool {
        let q = Quadrant::from_pos(pos);
        let pos = pos.re_center(self.half_width() / 2);
        match self.depth_quad() {
            DepthQuad::Leaf(leaf) => leaf[q].to_rows() & block_pos_to_mask(pos) != 0,
            DepthQuad::Inner(_, inner) => inner[q].get_in_bounds(pos),
        }
    }

    pub fn set(&self, pos: Pos, alive: bool) -> Self {
        // TODO use center_at_depth
        let half_width = self.half_width();
        if pos.x >= half_width || pos.y >= half_width || pos.x < -half_width || pos.y < -half_width
        {
            self.expand().set(pos, alive)
        } else {
            self.set_in_bounds(pos, alive)
        }
    }
    fn set_in_bounds(&self, pos: Pos, alive: bool) -> Self {
        let q = Quadrant::from_pos(pos);
        let pos = pos.re_center(self.half_width() / 2);
        match self.depth_quad() {
            DepthQuad::Leaf(leaf) => {
                let mut leaf = *leaf;
                let mut child = leaf[q].to_rows();
                let mask = block_pos_to_mask(pos);
                if alive {
                    child |= mask;
                } else {
                    child &= !mask;
                }
                leaf[q] = Block::from_rows(child);
                Node::from(leaf)
            }
            DepthQuad::Inner(depth, inner) => {
                let mut inner = inner.clone();
                inner[q] = inner[q].set_in_bounds(pos, alive);
                Node::new_depth_inner(*depth, inner)
            }
        }
    }
}
fn block_pos_to_mask(pos: Pos) -> u64 {
    // pos is in [-4,3]x[-4,3], so linearized is [-36,27]
    // but bit order is reversed, so 27 - linearized
    1 << (27 - (pos.y * 8 + pos.x))
}
