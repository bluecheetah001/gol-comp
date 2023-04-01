//! Raw types to name indicies to what would otherwise be fixed length arrays

use std::fmt::Debug;
use std::ops::{Add, Neg, Sub};

use crate::Quadrant;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
/// x increases to the east
/// y increases to the south
pub struct Pos {
    pub x: i64,
    pub y: i64,
}
impl Pos {
    pub fn in_dir(q: Quadrant, dist: i64) -> Self {
        Self {
            x: if q.is_west() { -dist } else { dist },
            y: if q.is_north() { -dist } else { dist },
        }
    }
    /// adds given amount to self to be closer to 0,0
    pub(crate) fn re_center(self, amount: i64) -> Self {
        Self {
            x: self.x + if self.x < 0 { amount } else { -amount },
            y: self.y + if self.y < 0 { amount } else { -amount },
        }
    }
    pub fn map<U>(self, mut f: impl FnMut(i64) -> i64) -> Self {
        Self {
            x: f(self.x),
            y: f(self.y),
        }
    }
}
impl Add for Pos {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Sub for Pos {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl Neg for Pos {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}
