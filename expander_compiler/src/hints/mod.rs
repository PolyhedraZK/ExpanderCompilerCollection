//! Module for handling hints in the expander compiler.

pub mod builtin;
pub mod registry;

pub use builtin::*;

use registry::HintCaller;

use crate::{field::Field, utils::error::Error};

/// Safely calls a hint implementation.
/// If the hint ID corresponds to a built-in hint, it calls the built-in implementation.
/// Otherwise, it calls the provided `HintCaller` implementation.
pub fn safe_impl<F: Field>(
    hint_caller: &mut impl HintCaller<F>,
    hint_id: usize,
    inputs: &[F],
    num_outputs: usize,
) -> Result<Vec<F>, Error> {
    match BuiltinHintIds::from_usize(hint_id) {
        Some(hint_id) => Ok(impl_builtin_hint(hint_id, inputs, num_outputs)),
        None => hint_caller.call(hint_id, inputs, num_outputs),
    }
}
