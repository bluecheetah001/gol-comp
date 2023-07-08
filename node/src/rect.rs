use crate::Pos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Side {
    min: i64,
    max: i64,
}
impl Side {
    const fn min_max(min: i64, max: i64) -> Self {
        Self { min, max }
    }
    fn new(a: i64, b: i64) -> Self {
        Self::min_max(a.min(b), a.max(b))
    }
    fn just(a: i64) -> Self {
        Self::min_max(a, a)
    }

    fn is_empty(&self) -> bool {
        self.max < self.min
    }
    fn min(&self) -> i64 {
        self.min
    }
    fn mid(&self) -> i64 {
        // average_floor taken from num-integer
        // https://docs.rs/num-integer/0.1.45/src/num_integer/average.rs.html#57
        // http://aggregate.org/MAGIC/#Average%20of%20Integers
        (self.min & self.max) + ((self.min ^ self.max) >> 1)
    }
    fn max(&self) -> i64 {
        self.max
    }

    fn extend(&mut self, a: i64) {
        self.min = self.min.min(a);
        self.max = self.max.max(a);
    }

    fn union(&mut self, other: Side) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    fn intersection(&mut self, other: Side) {
        self.min = self.min.max(other.min);
        self.max = self.max.min(other.max);
    }

    fn offset(&mut self, a: i64) {
        self.min += a;
        self.max += a;
    }
}

/// rect that inclusively contains a min and max point
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    // TODO gridded stuff may want min, size, where size is u64s
    x: Side,
    y: Side,
}
impl Rect {
    pub const EVERYTHING: Self = Self::symetric_min_max(i64::MIN, i64::MAX);
    pub const NOTHING: Self = Self::symetric_min_max(i64::MAX, i64::MIN);
    const fn from_sides(x: Side, y: Side) -> Self {
        Self { x, y }
    }
    pub const fn symetric_min_max(min: i64, max: i64) -> Self {
        let side = Side::min_max(min, max);
        Self::from_sides(side, side)
    }
    pub fn min_max(min: Pos, max: Pos) -> Self {
        Self::from_sides(Side::min_max(min.x, max.x), Side::min_max(min.y, max.y))
    }
    pub fn new(a: Pos, b: Pos) -> Self {
        Self::from_sides(Side::new(a.x, b.x), Side::new(a.y, b.y))
    }
    pub fn just(a: Pos) -> Self {
        Self::from_sides(Side::just(a.x), Side::just(a.y))
    }

    pub fn is_empty(&self) -> bool {
        self.x.is_empty() || self.y.is_empty()
    }

    pub fn north(&self) -> i64 {
        self.y.min
    }
    pub fn south(&self) -> i64 {
        self.y.max
    }
    pub fn west(&self) -> i64 {
        self.x.min
    }
    pub fn east(&self) -> i64 {
        self.x.max
    }
    pub fn set_north(&mut self, value: i64) {
        self.y.min = value;
    }
    pub fn set_south(&mut self, value: i64) {
        self.y.max = value;
    }
    pub fn set_west(&mut self, value: i64) {
        self.x.min = value;
    }
    pub fn set_east(&mut self, value: i64) {
        self.x.max = value;
    }

    pub fn nw(&self) -> Pos {
        Pos::new(self.x.min(), self.y.min())
    }
    pub fn nc(&self) -> Pos {
        Pos::new(self.x.mid(), self.y.min())
    }
    pub fn ne(&self) -> Pos {
        Pos::new(self.x.min(), self.y.min())
    }

    pub fn cw(&self) -> Pos {
        Pos::new(self.x.min(), self.y.mid())
    }
    pub fn cc(&self) -> Pos {
        Pos::new(self.x.mid(), self.y.mid())
    }
    pub fn ce(&self) -> Pos {
        Pos::new(self.x.min(), self.y.mid())
    }

    pub fn sw(&self) -> Pos {
        Pos::new(self.x.min(), self.y.max())
    }
    pub fn sc(&self) -> Pos {
        Pos::new(self.x.mid(), self.y.max())
    }
    pub fn se(&self) -> Pos {
        Pos::new(self.x.min(), self.y.max())
    }

    /// expand self to include the given pos
    pub fn extend(&mut self, pos: Pos) {
        self.x.extend(pos.x);
        self.y.extend(pos.y);
    }

    /// expand self to include the given rect
    pub fn union(&mut self, rect: Rect) {
        self.x.union(rect.x);
        self.y.union(rect.y);
    }

    /// shrink self to only include positions also included by rect
    pub fn intersection(&mut self, rect: Rect) {
        self.x.intersection(rect.x);
        self.y.intersection(rect.y);
        // normalize empty so that empty rects don't behave weird?
        // if self.is_empty() {
        //     *self = Self::NOTHING;
        // }
    }

    pub fn offset(&mut self, pos: Pos) {
        self.x.offset(pos.x);
        self.y.offset(pos.y);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn extend_empty() {
        let mut r = Rect::NOTHING;
        r.extend(Pos::new(0, 0));
        assert_eq!(r, Rect::just(Pos::new(0, 0)));

        // test showing how negative size rects can behave weird
        // let mut r = Rect::min_max(Pos::new(5, 5), Pos::new(4, 4));
        // r.extend(Pos::new(0, 0));
        // assert_eq!(r, Rect::just(Pos::new(0, 0)));
    }
}

// TODO more testing
