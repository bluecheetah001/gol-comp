//! Raw types to name indicies to what would otherwise be fixed length arrays

use std::fmt::Debug;
use std::iter::FusedIterator;
use std::num::NonZeroU8;
use std::ops::{Index, IndexMut};

use crate::pos::Pos;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Quadrant {
    NW,
    NE,
    SW,
    SE,
}
impl Quadrant {
    pub fn from_pos(pos: &Pos) -> Self {
        if pos.y < 0 {
            if pos.x < 0 {
                Self::NW
            } else {
                Self::NE
            }
        } else {
            if pos.x < 0 {
                Self::SW
            } else {
                Self::SE
            }
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Self::NW => Self::SE,
            Self::NE => Self::SW,
            Self::SW => Self::NE,
            Self::SE => Self::NW,
        }
    }

    pub fn iter_all() -> impl Iterator<Item = Quadrant> {
        QuadrantIter::new()
    }
}
struct QuadrantIter {
    next: Option<Quadrant>,
}
impl QuadrantIter {
    pub fn new() -> Self {
        QuadrantIter {
            next: Some(Quadrant::NW),
        }
    }
}
impl Iterator for QuadrantIter {
    type Item = Quadrant;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.next;
        let next = match curr {
            Some(Quadrant::NW) => Some(Quadrant::NE),
            Some(Quadrant::NE) => Some(Quadrant::SW),
            Some(Quadrant::SW) => Some(Quadrant::SE),
            _ => None,
        };
        self.next = next;
        curr
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.len();
        (size, Some(size))
    }
}
impl FusedIterator for QuadrantIter {}
impl ExactSizeIterator for QuadrantIter {
    fn len(&self) -> usize {
        match self.next {
            Some(Quadrant::NW) => 4,
            Some(Quadrant::NE) => 3,
            Some(Quadrant::SW) => 2,
            Some(Quadrant::SE) => 1,
            None => 0,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Quad<T> {
    pub nw: T,
    pub ne: T,
    pub sw: T,
    pub se: T,
}
impl<T> Quad<T> {
    pub fn as_ref(&self) -> Quad<&T> {
        Quad {
            nw: &self.nw,
            ne: &self.ne,
            sw: &self.sw,
            se: &self.se,
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        Quadrant::iter_all().map(|q| &self[q])
    }
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Quad<U> {
        Quad {
            nw: f(self.nw),
            ne: f(self.ne),
            sw: f(self.sw),
            se: f(self.se),
        }
    }
    pub fn index_map<U>(self, mut f: impl FnMut(Quadrant, T) -> U) -> Quad<U> {
        Quad {
            nw: f(Quadrant::NW, self.nw),
            ne: f(Quadrant::NE, self.ne),
            sw: f(Quadrant::SW, self.sw),
            se: f(Quadrant::SE, self.se),
        }
    }
    // pub fn zip_map<U, V>(self, other: Quad<U>, mut f: impl FnMut(T, U) -> V) -> Quad<V> {
    //     Quad {
    //         nw: f(self.nw, other.nw),
    //         ne: f(self.nw, other.nw),
    //         sw: f(self.nw, other.nw),
    //         se: f(self.nw, other.nw),
    //     }
    // }
    // pub fn zip<U>(self, other: Quad<U>) -> Quad<(T, U)> {
    //     self.zip_map(other, |a, b| (a, b))
    // }
    pub fn expand(self, empty: T) -> Quad<Quad<T>>
    where
        T: Clone,
    {
        Quad {
            nw: Quad {
                nw: empty.clone(),
                ne: empty.clone(),
                sw: empty.clone(),
                se: self.nw,
            },
            ne: Quad {
                nw: empty.clone(),
                ne: empty.clone(),
                sw: self.ne,
                se: empty.clone(),
            },
            sw: Quad {
                nw: empty.clone(),
                ne: self.sw,
                sw: empty.clone(),
                se: empty.clone(),
            },
            se: Quad {
                nw: self.se,
                ne: empty.clone(),
                sw: empty.clone(),
                se: empty,
            },
        }
    }
}
impl<T> IntoIterator for Quad<T> {
    type Item = T;
    type IntoIter = std::array::IntoIter<T, 4>;
    fn into_iter(self) -> Self::IntoIter {
        [self.nw, self.ne, self.sw, self.se].into_iter()
    }
}
impl<'a, T> IntoIterator for &'a Quad<T> {
    type Item = &'a T;
    type IntoIter = impl Iterator<Item = &'a T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
// impl<T: Copy> Quad<&T> {
//     pub fn copied(self) -> Quad<T> {
//         self.map(|v| *v)
//     }
// }
// impl<T: Clone> Quad<&T> {
//     pub fn cloned(self) -> Quad<T> {
//         self.map(Clone::clone)
//     }
// }
impl<T> Index<Quadrant> for Quad<T> {
    type Output = T;
    fn index(&self, index: Quadrant) -> &Self::Output {
        match index {
            Quadrant::NW => &self.nw,
            Quadrant::NE => &self.ne,
            Quadrant::SW => &self.sw,
            Quadrant::SE => &self.se,
        }
    }
}
impl<T> IndexMut<Quadrant> for Quad<T> {
    fn index_mut(&mut self, index: Quadrant) -> &mut Self::Output {
        match index {
            Quadrant::NW => &mut self.nw,
            Quadrant::NE => &mut self.ne,
            Quadrant::SW => &mut self.sw,
            Quadrant::SE => &mut self.se,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DepthQuad<L, I> {
    Leaf(Quad<L>),
    Inner(NonZeroU8, Quad<I>),
}
impl<L, I> DepthQuad<L, I> {
    pub fn depth(&self) -> u8 {
        match self {
            Self::Leaf(_) => 0,
            Self::Inner(depth, _) => depth.get(),
        }
    }
    pub fn leaf(&self) -> Option<&Quad<L>> {
        match self {
            Self::Leaf(leaf) => Some(leaf),
            Self::Inner(_, _) => None,
        }
    }
    pub fn inner(&self) -> Option<&Quad<I>> {
        match self {
            Self::Leaf(_) => None,
            Self::Inner(_, inner) => Some(inner),
        }
    }
}
