use std::collections::HashMap;

use tiny_keccak::Hasher;

use crate::{field::Field, utils::error::Error};

use super::BuiltinHintIds;

pub type HintFn<F> = dyn FnMut(&[F], &mut [F]) -> Result<(), Error>;

pub struct HintRegistry<F: Field> {
    hints: HashMap<usize, Box<HintFn<F>>>,
}

pub fn hint_key_to_id(key: &str) -> usize {
    let mut hasher = tiny_keccak::Keccak::v256();
    hasher.update(key.as_bytes());
    let mut hash = [0u8; 32];
    hasher.finalize(&mut hash);

    let res = usize::from_le_bytes(hash[0..8].try_into().unwrap());
    if BuiltinHintIds::from_usize(res).is_some() {
        panic!("Hint id {} collides with a builtin hint id", res);
    }
    res
}

impl<F: Field> HintRegistry<F> {
    pub fn new() -> Self {
        Self {
            hints: HashMap::new(),
        }
    }
    pub fn register<Hint: Fn(&[F], &mut [F]) -> Result<(), Error> + 'static>(
        &mut self,
        key: &str,
        hint: Hint,
    ) {
        let id = hint_key_to_id(key);
        if self.hints.contains_key(&id) {
            panic!("Hint with id {} already exists", id);
        }
        self.hints.insert(id, Box::new(hint));
    }
    pub fn call(&mut self, id: usize, args: &[F], num_outputs: usize) -> Result<Vec<F>, Error> {
        if let Some(hint) = self.hints.get_mut(&id) {
            let mut outputs = vec![F::zero(); num_outputs];
            hint(args, &mut outputs).map(|_| outputs)
        } else {
            panic!("Hint with id {} not found", id);
        }
    }
}
