//! Raw types to name indicies to what would otherwise be fixed length arrays

use std::fmt::Debug;
use std::ops::{Add, Sub};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
/// x increases to the east
/// y increases to the south
pub struct Pos {
    pub x: i64,
    pub y: i64,
}
impl Pos {
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
