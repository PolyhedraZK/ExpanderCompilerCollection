//! This module contains the `InputMapping` struct, which is used to map inputs.
//! In compilation, some inputs may be removed, and the mapping is used to
//! ensure that the remaining inputs are correctly mapped to their new positions.

use serdes::ExpSerde;

/// The `EMPTY` constant represents an unused position in the mapping.
pub const EMPTY: usize = usize::MAX >> 9;

/// The `InputMapping` struct is used to map inputs from one size to another.
/// It ensures `mapped_input[mapping[i]] == input[i]` for all valid `i`.
/// If `mapping[i]` is `EMPTY`, it means that the input at position `i` is removed
/// during the mapping process.
/// The `next_size` field indicates the size of the next input vector after mapping.
#[derive(Debug, Clone, Hash, PartialEq, Eq, ExpSerde)]
pub struct InputMapping {
    next_size: usize,
    mapping: Vec<usize>,
}

impl InputMapping {
    /// Creates a new `InputMapping` with the specified `next_size` and `mapping`.
    pub fn new(next_size: usize, mapping: Vec<usize>) -> Self {
        InputMapping { next_size, mapping }
    }

    /// Creates a new `InputMapping` that is an identity mapping for the given `next_size`.
    pub fn new_identity(next_size: usize) -> Self {
        InputMapping {
            next_size,
            mapping: (0..next_size).collect(),
        }
    }

    /// Returns the current size of the mapping, which is the length of the `mapping` vector.
    pub fn cur_size(&self) -> usize {
        self.mapping.len()
    }

    /// Returns the next size of the mapping, which is the `next_size` field.
    pub fn next_size(&self) -> usize {
        self.next_size
    }

    /// Returns the mapping for a given position.
    pub fn map(&self, pos: usize) -> usize {
        self.mapping[pos]
    }

    /// Maps the inputs according to the mapping defined in this `InputMapping`.
    pub fn map_inputs<T: Default + Clone>(&self, inputs: &[T]) -> Vec<T> {
        assert_eq!(inputs.len(), self.mapping.len());
        let mut new_inputs = vec![T::default(); self.next_size];
        for i in 0..inputs.len() {
            if self.mapping[i] != EMPTY {
                new_inputs[self.mapping[i]] = inputs[i].clone();
            }
        }
        new_inputs
    }

    /// Returns a reference to the mapping vector.
    pub fn mapping(&self) -> &Vec<usize> {
        &self.mapping
    }

    /// Validates the `InputMapping` to ensure that it is a valid mapping.
    pub fn validate(&self) -> bool {
        let mut used = vec![false; self.next_size];
        for &m in &self.mapping {
            if m != EMPTY {
                if m >= self.next_size {
                    return false;
                }
                if used[m] {
                    return false;
                }
                used[m] = true;
            }
        }
        for &u in &used {
            if !u {
                return false;
            }
        }
        true
    }

    /// Composes this `InputMapping` with another `InputMapping`.
    /// The resulting mapping is `other(self(inputs))`.
    pub fn compose(&self, other: &InputMapping) -> InputMapping {
        let mut new_mapping = Vec::new();
        for i in 0..self.mapping.len() {
            if self.mapping[i] == EMPTY {
                new_mapping.push(EMPTY);
            } else {
                new_mapping.push(other.mapping[self.mapping[i]]);
            }
        }
        InputMapping {
            next_size: other.next_size,
            mapping: new_mapping,
        }
    }

    /// Composes this `InputMapping` with another `InputMapping` in place.
    pub fn compose_in_place(&mut self, other: &InputMapping) {
        for i in 0..self.mapping.len() {
            if self.mapping[i] != EMPTY {
                self.mapping[i] = other.mapping[self.mapping[i]];
            }
        }
        self.next_size = other.next_size;
    }
}
