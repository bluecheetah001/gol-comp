#![feature(let_chains)]
#![feature(const_option)]
#![feature(local_key_cell_methods)]
#![feature(assert_matches)]
#![feature(type_alias_impl_trait)]
#![feature(ptr_sub_ptr)]
#![feature(iter_array_chunks)]
#![feature(int_roundings)]
#![feature(new_uninit)]
// lints
#![warn(clippy::pedantic)]
#![allow(
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::items_after_statements,
    clippy::module_name_repetitions,
    clippy::manual_range_contains,
    clippy::bool_to_int_with_if,
    clippy::collapsible_else_if,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc // TODO remove at some point
)]

mod block;
mod node;
mod pos;
mod quad;
mod ops {
    mod bit;
    mod center;
    mod children;
    mod get;
    mod mc_format;
    mod offset;
    mod offset_norm;
    mod population;
    mod reduce;
    mod step;
    mod test_format;

    pub use mc_format::*;
    pub use population::*;
    pub use test_format::*;
}

pub use crate::node::*;
pub use block::*;
pub use ops::*;
pub use pos::*;
pub use quad::*;
