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
        } else if rect == self.trivial_bounding_rect() {
            self.clone()
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
        } else if rect == self.trivial_bounding_rect() {
            Node::empty(self.depth())
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
        if rect.is_empty() {
            Self::empty()
        } else {
            let rows = self.to_rows();
            let mask = rect.to_block_rows();
            Self::from_rows(rows & mask)
        }
    }
    fn clear_in_bounds(self, rect: Rect) -> Block {
        if rect.is_empty() {
            self
        } else {
            let rows = self.to_rows();
            let mask = rect.to_block_rows();
            Self::from_rows(rows & !mask)
        }
    }
}
impl Rect {
    fn nw_shifted(&self, amount: i64) -> Rect {
        let mut rect = *self;
        rect.intersection(Rect::min_max(
            Pos::new(i64::MIN, i64::MIN),
            Pos::new(-1, -1),
        ));
        rect.offset(Pos::new(amount, amount));
        rect
    }
    fn ne_shifted(&self, amount: i64) -> Rect {
        let mut rect = *self;
        rect.intersection(Rect::min_max(Pos::new(0, i64::MIN), Pos::new(i64::MAX, -1)));
        rect.offset(Pos::new(-amount, amount));
        rect
    }
    fn sw_shifted(&self, amount: i64) -> Rect {
        let mut rect = *self;
        rect.intersection(Rect::min_max(Pos::new(i64::MIN, 0), Pos::new(-1, i64::MAX)));
        rect.offset(Pos::new(amount, -amount));
        rect
    }
    fn se_shifted(&self, amount: i64) -> Rect {
        let mut rect = *self;
        rect.intersection(Rect::min_max(Pos::new(0, 0), Pos::new(i64::MAX, i64::MAX)));
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

#[cfg(test)]
mod test {
    use crate::{Block, Node, Pos, Rect};

    #[test]
    pub fn centered() {
        let base = Node::new(
            Block::from_rows(0x01_01_01_01_01_01_01_ff),
            Block::from_rows(0x80_80_80_80_80_80_80_ff),
            Block::from_rows(0xff_01_01_01_01_01_01_01),
            Block::from_rows(0xff_80_80_80_80_80_80_80),
        );
        let actual = base.clip(Rect::new(Pos::new(-2, -3), Pos::new(4, 3)));
        let expected = Node::new(
            Block::from_rows(0x00_00_00_00_00_01_01_03),
            Block::from_rows(0x00_00_00_00_00_80_80_f8),
            Block::from_rows(0x03_01_01_01_00_00_00_00),
            Block::from_rows(0xf8_80_80_80_00_00_00_00),
        );
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn se_corner() {
        let base = Node::new(
            Block::from_rows(0xff_ff_ff_ff_ff_ff_ff_ff),
            Block::from_rows(0xff_ff_ff_ff_ff_ff_ff_ff),
            Block::from_rows(0xff_ff_ff_ff_ff_ff_ff_ff),
            Block::from_rows(0xff_ff_ff_ff_ff_ff_ff_ff),
        );
        let actual = base.clip(Rect::new(Pos::new(3, 2), Pos::new(6, 4)));
        let expected = Node::new(
            Block::from_rows(0x00_00_00_00_00_00_00_00),
            Block::from_rows(0x00_00_00_00_00_00_00_00),
            Block::from_rows(0x00_00_00_00_00_00_00_00),
            Block::from_rows(0x00_00_1e_1e_1e_00_00_00),
        );
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn big() {
        let leaf = Node::new(
            Block::from_rows(0x01_01_01_01_01_01_01_ff),
            Block::from_rows(0x80_80_80_80_80_80_80_ff),
            Block::from_rows(0xff_01_01_01_01_01_01_01),
            Block::from_rows(0xff_80_80_80_80_80_80_80),
        );
        let base = Node::new(leaf.clone(), leaf.clone(), leaf.clone(), leaf);
        let leaf_clip = Node::new(
            Block::from_rows(0x00_00_00_00_00_01_01_03),
            Block::from_rows(0x00_00_00_00_00_80_80_f8),
            Block::from_rows(0x03_01_01_01_00_00_00_00),
            Block::from_rows(0xf8_80_80_80_00_00_00_00),
        );
        assert_eq!(
            base.clip(Rect::new(Pos::new(-10, -11), Pos::new(-4, -5))),
            Node::new(
                leaf_clip.clone(),
                Node::empty(0),
                Node::empty(0),
                Node::empty(0)
            )
        );
        assert_eq!(
            base.clip(Rect::new(Pos::new(6, -11), Pos::new(12, -5))),
            Node::new(
                Node::empty(0),
                leaf_clip.clone(),
                Node::empty(0),
                Node::empty(0)
            )
        );
        assert_eq!(
            base.clip(Rect::new(Pos::new(-10, 5), Pos::new(-4, 11))),
            Node::new(
                Node::empty(0),
                Node::empty(0),
                leaf_clip.clone(),
                Node::empty(0)
            )
        );
        assert_eq!(
            base.clip(Rect::new(Pos::new(6, 5), Pos::new(12, 11))),
            Node::new(Node::empty(0), Node::empty(0), Node::empty(0), leaf_clip)
        );
    }
}
