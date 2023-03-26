#![feature(int_log)]
#![feature(let_chains)]
#![feature(const_option)]
#![feature(local_key_cell_methods)]
#![feature(assert_matches)]
#![feature(type_alias_impl_trait)]
#![feature(ptr_sub_ptr)]
#![feature(iter_array_chunks)]

mod block;
mod node;
mod pos;
mod quad;
mod ops {
    // mod get;
    mod center;
    mod children;
    mod mc_format;
    mod population;
    mod step;
    mod test_format;

    pub use mc_format::*;
    pub use population::*;
    pub use test_format::*;
}

pub use crate::node::Node;
pub use block::Block;
pub use ops::*;
pub use quad::{DepthQuad, Quad};