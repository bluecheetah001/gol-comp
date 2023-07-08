use crate::{Block, DepthQuad, Node, Pos, Quad, Rect};

impl Node {
    pub fn clip(&self, mut rect: Rect) -> Node {
        rect.intersection(self.trivial_bounding_rect());
        self.clip_in_bounds(rect)
    }
    pub fn clear(&self, mut rect: Rect) -> Node {
        rect.intersection(self.trivial_bounding_rect());
        self.clear_in_bounds(rect)
    }
    fn trivial_bounding_rect(&self) -> Rect {
        let half_width = self.half_width();
        Rect::symetric_min_max(-half_width, half_width - 1)
    }

    fn clip_in_bounds(&self, rect: Rect) -> Node {
        if rect.is_empty() {
            Node::empty(self.depth())
        } else {
            match self.depth_quad() {
                DepthQuad::Leaf(leaf) => Node::new_leaf(leaf.clip_in_bounds(rect)),
                DepthQuad::Inner(depth, inner) => {
                    Node::new_depth_inner(*depth, inner.clip_in_bounds(rect))
                }
            }
        }
    }
    fn clear_in_bounds(&self, rect: Rect) -> Node {
        if rect.is_empty() {
            self.clone()
        } else {
            match self.depth_quad() {
                DepthQuad::Leaf(leaf) => Node::new_leaf(leaf.clear_in_bounds(rect)),
                DepthQuad::Inner(depth, inner) => {
                    Node::new_depth_inner(*depth, inner.clear_in_bounds(rect))
                }
            }
        }
    }
}

impl Quad<Node> {
    fn clip_in_bounds(&self, rect: Rect) -> Quad<Node> {
        let amount = self.nw.half_width();
        Quad {
            nw: self.nw.clip_in_bounds(rect.nw_shifted(amount)),
            ne: self.ne.clip_in_bounds(rect.ne_shifted(amount)),
            sw: self.sw.clip_in_bounds(rect.sw_shifted(amount)),
            se: self.se.clip_in_bounds(rect.se_shifted(amount)),
        }
    }
    fn clear_in_bounds(&self, rect: Rect) -> Quad<Node> {
        let amount = self.nw.half_width();
        Quad {
            nw: self.nw.clear_in_bounds(rect.nw_shifted(amount)),
            ne: self.ne.clear_in_bounds(rect.ne_shifted(amount)),
            sw: self.sw.clear_in_bounds(rect.sw_shifted(amount)),
            se: self.se.clear_in_bounds(rect.se_shifted(amount)),
        }
    }
}
impl Quad<Block> {
    fn clip_in_bounds(&self, rect: Rect) -> Quad<Block> {
        let amount = Block::HALF_WIDTH;
        Quad {
            nw: self.nw.clip_in_bounds(rect.nw_shifted(amount)),
            ne: self.ne.clip_in_bounds(rect.ne_shifted(amount)),
            sw: self.sw.clip_in_bounds(rect.sw_shifted(amount)),
            se: self.se.clip_in_bounds(rect.se_shifted(amount)),
        }
    }
    fn clear_in_bounds(&self, rect: Rect) -> Quad<Block> {
        let amount = Block::HALF_WIDTH;
        Quad {
            nw: self.nw.clear_in_bounds(rect.nw_shifted(amount)),
            ne: self.ne.clear_in_bounds(rect.ne_shifted(amount)),
            sw: self.sw.clear_in_bounds(rect.sw_shifted(amount)),
            se: self.se.clear_in_bounds(rect.se_shifted(amount)),
        }
    }
}

impl Block {
    fn clip_in_bounds(self, rect: Rect) -> Block {
        let rows = self.to_rows();
        let mask = rect.to_block_rows();
        Self::from_rows(rows & mask)
    }
    fn clear_in_bounds(self, rect: Rect) -> Block {
        let rows = self.to_rows();
        let mask = rect.to_block_rows();
        Self::from_rows(rows & !mask)
    }
}
impl Rect {
    fn nw_shifted(&self, amount: i64) -> Rect {
        let mut rect = *self;
        rect.set_east(-1);
        rect.set_south(-1);
        rect.offset(Pos::new(amount, amount));
        rect
    }
    fn ne_shifted(&self, amount: i64) -> Rect {
        let mut rect = *self;
        rect.set_west(0);
        rect.set_south(-1);
        rect.offset(Pos::new(-amount, amount));
        rect
    }
    fn sw_shifted(&self, amount: i64) -> Rect {
        let mut rect = *self;
        rect.set_east(-1);
        rect.set_north(0);
        rect.offset(Pos::new(amount, -amount));
        rect
    }
    fn se_shifted(&self, amount: i64) -> Rect {
        let mut rect = *self;
        rect.set_west(0);
        rect.set_north(0);
        rect.offset(Pos::new(-amount, -amount));
        rect
    }
    fn to_block_rows(self) -> u64 {
        assert!(self.west() >= -4);
        assert!(self.east() < 4);
        assert!(self.east() >= self.west());

        assert!(self.north() >= -4);
        assert!(self.south() < 4);
        assert!(self.south() >= self.north());

        let row_mask_high = 1_u8 << (3 - self.west());
        let row_mask_low = 1_u8 << (3 - self.east());
        // could simplify if 1_u8<<8 is defined as 0 and explicitly use wrapping sub
        #[allow(clippy::cast_lossless)] // all types are local
        let row_mask = (row_mask_high | (row_mask_high - row_mask_low)) as u64;
        let row_mask = row_mask
            | (row_mask << 8)
            | (row_mask << 16)
            | (row_mask << 24)
            | (row_mask << 32)
            | (row_mask << 40)
            | (row_mask << 48)
            | (row_mask << 56);

        let col_mask_high = 1_u64 << (31 - self.north() * 8);
        let col_mask_low = 1_u64 << (24 - self.south() * 8);
        // could simplify if 1_u64<<64 is defined as 0 and explicitly use wrapping sub
        let col_mask = col_mask_high | (col_mask_high - col_mask_low);

        row_mask & col_mask
    }
}

// TODO testing!
