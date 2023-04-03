use crate::{Block, DepthQuad, Quad};

// TODO this is nice as a trait because it is used a decent amount, but annoying to have to bring it into scope
pub trait Population {
    /// if population returns `u64::MAX` the actual population may be larger
    fn population(&self) -> u64;
    fn is_empty(&self) -> bool;
}
impl<'a, T> Population for &'a T
where
    T: Population,
{
    fn population(&self) -> u64 {
        (**self).population()
    }
    fn is_empty(&self) -> bool {
        (**self).is_empty()
    }
}

impl Population for Block {
    fn population(&self) -> u64 {
        self.to_rows().count_ones().into()
    }
    fn is_empty(&self) -> bool {
        self.to_rows() == 0
    }
}

impl<T> Population for Quad<T>
where
    T: Population,
{
    fn population(&self) -> u64 {
        self.iter().map(T::population).fold(0, u64::saturating_add)
    }
    fn is_empty(&self) -> bool {
        self.iter().all(T::is_empty)
    }
}

impl<L, I> Population for DepthQuad<L, I>
where
    L: Population,
    I: Population,
{
    fn population(&self) -> u64 {
        match self {
            DepthQuad::Leaf(leaf) => leaf.population(),
            DepthQuad::Inner(_, inner) => inner.population(),
        }
    }
    fn is_empty(&self) -> bool {
        match self {
            DepthQuad::Leaf(leaf) => leaf.is_empty(),
            DepthQuad::Inner(_, inner) => inner.is_empty(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Block, Node, Population};

    #[test]
    fn block() {
        assert_eq!(0, Block::empty().population());
        assert!(Block::empty().is_empty());
        assert_eq!(1, Block::from_rows(0x80_00_00_00_00_00_00_00).population());
        assert!(!Block::from_rows(0x80_00_00_00_00_00_00_00).is_empty());
        assert_eq!(8, Block::from_rows(0x00_00_00_00_00_00_ff_00).population());
        assert!(!Block::from_rows(0x00_00_00_00_00_00_ff_00).is_empty());
    }

    #[test]
    fn node() {
        assert_eq!(0, Node::empty(5).population());
        assert!(Node::empty(7).is_empty());

        let b1 = Block::from_rows(0x01_01_01_01_01_01_01_01);
        let b2 = Block::from_rows(0x80_40_20_10_08_04_02_01);
        let b3 = Block::from_rows(0xff_ee_dd_cc_bb_aa_99_88);
        let b4 = Block::empty();
        let mut n = Node::new(b1, b2, b3, b4);
        assert_eq!(56, n.population());
        let pops = [224, 896, 3584, 14336]; // TODO add more
        for p in pops {
            n = Node::new(n.clone(), n.clone(), n.clone(), n);
            assert_eq!(p, n.population());
        }
    }
}
