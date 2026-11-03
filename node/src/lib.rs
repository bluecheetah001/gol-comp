#![feature(iter_array_chunks)]
#![feature(int_roundings)]
#![feature(impl_trait_in_assoc_type)]
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
mod rect;
mod ops {
    mod bit;
    mod center;
    mod children;
    mod clip;
    mod get;
    mod mc_format;
    mod offset;
    mod offset_norm;
    mod population;
    mod reduce;
    mod rotate;
    mod step;
    mod test_format;

    pub use mc_format::*;
    pub use population::*;
}

pub use crate::node::*;
pub use block::*;
pub use ops::*;
pub use pos::*;
pub use quad::*;
pub use rect::*;
