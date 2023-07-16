use crate::{Block, DepthQuad, Node, Pos, Quad};

// TODO memoize? or if not then special case empty

// TODO if this was normalizing then offset_norm wouldn't be as necessary

impl Node {
    #[allow(clippy::cast_sign_loss)] // is checked for
    pub fn offset(&self, amount: Pos) -> Self {
        let max_offset = self.half_width();
        // TODO use center_at_depth
        // note that this is > +max, where as set is >= +max
        // this is taking a delta, where as set is taking an absolute
        if amount.x < -max_offset
            || amount.x > max_offset
            || amount.y < -max_offset
            || amount.y > max_offset
        {
            return self.expand().offset(amount);
        }

        let width = self.width();
        let (x0, xc) = if amount.x < 0 {
            (1, width.wrapping_add_signed(amount.x))
        } else {
            (0, amount.x as u64)
        };
        let (y0, yc) = if amount.y < 0 {
            (1, width.wrapping_add_signed(amount.y))
        } else {
            (0, amount.y as u64)
        };
        let empty = Node::empty(self.depth());
        let quad = self.expand_quad();
        // TODO this duplicates Quad<&Quad<Node|Block>>::offset_shrink_se
        // probably could be cleaned up a good bit
        let as_array = [
            [&empty, &empty, &empty, &empty],
            [&empty, &quad.nw, &quad.ne, &empty],
            [&empty, &quad.sw, &quad.se, &empty],
            [&empty, &empty, &empty, &empty],
        ];
        let offset_child = move |xi: usize, yi: usize| -> Node {
            Quad {
                nw: as_array[yi][xi],
                ne: as_array[yi][xi + 1],
                sw: as_array[yi + 1][xi],
                se: as_array[yi + 1][xi + 1],
            }
            .offset_shrink_se(xc, yc)
        };

        Quad {
            nw: offset_child(x0, y0),
            ne: offset_child(x0 + 1, y0),
            sw: offset_child(x0, y0 + 1),
            se: offset_child(x0 + 1, y0 + 1),
        }
        .into()
    }
}

impl Quad<&Node> {
    fn offset_shrink_se(self, x: u64, y: u64) -> Node {
        if x == 0 && y == 0 {
            return self.se.clone();
        }
        match self.children() {
            DepthQuad::Leaf(leaf) => leaf.offset_shrink_se(x, y).into(),
            DepthQuad::Inner(_, inner) => inner.offset_shrink_se(x, y).into(),
        }
    }
}

impl Quad<&Quad<Node>> {
    #[allow(clippy::cast_possible_truncation)] // only need first bit
    fn offset_shrink_se(self, x: u64, y: u64) -> Quad<Node> {
        let bit = self.nw.nw.width_log2();

        let mask = (1 << bit) - 1;
        let xc = x & mask;
        let yc = y & mask;
        let as_array = [
            [&self.nw.nw, &self.nw.ne, &self.ne.nw, &self.ne.ne],
            [&self.nw.sw, &self.nw.se, &self.ne.sw, &self.ne.se],
            [&self.sw.nw, &self.sw.ne, &self.se.nw, &self.se.ne],
            [&self.sw.sw, &self.sw.se, &self.se.sw, &self.se.se],
        ];
        let offset_child = move |xi: usize, yi: usize| -> Node {
            Quad {
                nw: as_array[yi][xi],
                ne: as_array[yi][xi + 1],
                sw: as_array[yi + 1][xi],
                se: as_array[yi + 1][xi + 1],
            }
            .offset_shrink_se(xc, yc)
        };

        // if we only need to shift a little, then index is 1 to use bits already in se
        // otherwise index is 0 to use bits in nw
        let x0 = (!x >> bit) as usize & 1;
        let y0 = (!y >> bit) as usize & 1;
        Quad {
            nw: offset_child(x0, y0),
            ne: offset_child(x0 + 1, y0),
            sw: offset_child(x0, y0 + 1),
            se: offset_child(x0 + 1, y0 + 1),
        }
    }
}

impl Quad<&Quad<Block>> {
    // shrinks and clips a 16x16 quad into an 8x8 block after shifting by 0..16 in SE direction
    #[allow(clippy::cast_possible_truncation)] // only need first bit
    fn offset_shrink_se(self, x: u64, y: u64) -> Quad<Block> {
        let bit = Block::WIDTH_LOG2;

        let mask = (1 << bit) - 1;
        let xc = x & mask;
        let yc = y & mask;
        let as_array = [
            [self.nw.nw, self.nw.ne, self.ne.nw, self.ne.ne],
            [self.nw.sw, self.nw.se, self.ne.sw, self.ne.se],
            [self.sw.nw, self.sw.ne, self.se.nw, self.se.ne],
            [self.sw.sw, self.sw.se, self.se.sw, self.se.se],
        ];
        let offset_child = move |xi: usize, yi: usize| -> Block {
            Quad {
                nw: as_array[yi][xi],
                ne: as_array[yi][xi + 1],
                sw: as_array[yi + 1][xi],
                se: as_array[yi + 1][xi + 1],
            }
            .offset_shrink_se(xc, yc)
        };

        // if we only need to shift a little, then index is 1 to use bits already in se
        // otherwise index is 0 to use bits in nw
        let x0 = (!x >> bit) as usize & 1;
        let y0 = (!y >> bit) as usize & 1;
        Quad {
            nw: offset_child(x0, y0),
            ne: offset_child(x0 + 1, y0),
            sw: offset_child(x0, y0 + 1),
            se: offset_child(x0 + 1, y0 + 1),
        }
    }
}

impl Quad<Block> {
    // shrinks and clips a 16x16 quad into an 8x8 block after shifting by 0..8 in SE direction
    fn offset_shrink_se(self, x: u64, y: u64) -> Block {
        fn offset_h(w: u64, e: u64, amount: u64) -> u64 {
            let w_mask = match amount {
                0 => return e,
                1 => 0x01_01_01_01_01_01_01_01,
                2 => 0x03_03_03_03_03_03_03_03,
                3 => 0x07_07_07_07_07_07_07_07,
                4 => 0x0f_0f_0f_0f_0f_0f_0f_0f,
                5 => 0x1f_1f_1f_1f_1f_1f_1f_1f,
                6 => 0x3f_3f_3f_3f_3f_3f_3f_3f,
                7 => 0x7f_7f_7f_7f_7f_7f_7f_7f,
                _ => panic!("offset x invalid"),
            };
            let w = (w & w_mask) << (8 - amount);
            let e = (e & !w_mask) >> amount;
            w | e
        }
        fn offset_v(n: u64, s: u64, amount: u64) -> u64 {
            if amount == 0 {
                return s;
            }
            assert!(amount < 8, "offset y invalid");
            let n = n << (64 - amount * 8);
            let s = s >> (amount * 8);
            n | s
        }
        let Quad { nw, ne, sw, se } = self.map(Block::to_rows);
        Block::from_rows(offset_h(offset_v(nw, sw, y), offset_v(ne, se, y), x))
    }
}

#[cfg(test)]
mod test {
    use crate::{Block, Node, Pos};

    #[test]
    fn small() {
        let outline = Node::new(
            Block::from_rows(0xff_80_80_80_80_80_80_80),
            Block::from_rows(0xff_01_01_01_01_01_01_01),
            Block::from_rows(0x80_80_80_80_80_80_80_ff),
            Block::from_rows(0x01_01_01_01_01_01_01_ff),
        );
        let n = Node::new(
            Node::new(
                Block::empty(),
                Block::from_rows(0x00_00_00_00_00_00_00_ff),
                Block::empty(),
                Block::from_rows(0x80_80_80_80_80_80_80_80),
            ),
            Node::new(
                Block::from_rows(0x00_00_00_00_00_00_00_ff),
                Block::empty(),
                Block::from_rows(0x01_01_01_01_01_01_01_01),
                Block::empty(),
            ),
            Node::new(
                Block::empty(),
                Block::from_rows(0x80_80_80_80_80_80_ff_00),
                Block::empty(),
                Block::empty(),
            ),
            Node::new(
                Block::from_rows(0x01_01_01_01_01_01_ff_00),
                Block::empty(),
                Block::empty(),
                Block::empty(),
            ),
        );
        let s = Node::new(
            Node::new(
                Block::empty(),
                Block::empty(),
                Block::empty(),
                Block::from_rows(0x00_ff_80_80_80_80_80_80),
            ),
            Node::new(
                Block::empty(),
                Block::empty(),
                Block::from_rows(0x00_ff_01_01_01_01_01_01),
                Block::empty(),
            ),
            Node::new(
                Block::empty(),
                Block::from_rows(0x80_80_80_80_80_80_80_80),
                Block::empty(),
                Block::from_rows(0xff_00_00_00_00_00_00_00),
            ),
            Node::new(
                Block::from_rows(0x01_01_01_01_01_01_01_01),
                Block::empty(),
                Block::from_rows(0xff_00_00_00_00_00_00_00),
                Block::empty(),
            ),
        );

        let s_actual = outline.offset(Pos::new(0, 1));
        assert_eq!(s_actual.inner(), s.inner(), "south");

        let n_actual = outline.offset(Pos::new(0, -1));
        assert_eq!(n_actual.inner(), n.inner(), "north");
    }
}
