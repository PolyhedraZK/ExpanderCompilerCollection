use std::collections::HashMap;

use tiny_keccak::Hasher;

use crate::{field::Field, utils::error::Error};

use super::{stub_impl, BuiltinHintIds};

type HintFn<F> = fn(&[F], &mut [F]) -> Result<(), Error>;

#[derive(Default, Clone)]
pub struct HintRegistry<F: Field> {
    hints: HashMap<usize, Box<HintFn<F>>>,
    custom_gate_type_to_hint_id: HashMap<usize, usize>,
}

pub fn hint_key_to_id(key: &str) -> usize {
    let mut hasher = tiny_keccak::Keccak::v256();
    hasher.update(key.as_bytes());
    let mut hash = [0u8; 32];
    hasher.finalize(&mut hash);

    let res = usize::from_le_bytes(hash[0..8].try_into().unwrap());
    if BuiltinHintIds::from_usize(res).is_some() {
        panic!("Hint id {res} collides with a builtin hint id");
    }
    res
}

impl<F: Field> HintRegistry<F> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn register(&mut self, key: &str, hint: fn(&[F], &mut [F]) -> Result<(), Error>) {
        let id = hint_key_to_id(key);
        if self.hints.contains_key(&id) {
            panic!("Hint with id {id} already exists");
        }
        self.hints.insert(id, hint);
    }
    pub fn register_custom_gate(&mut self, gate_type: usize, key: &str) {
        // TODO: check
        let id = hint_key_to_id(key);
        self.custom_gate_type_to_hint_id.insert(gate_type, id);
    }
    pub fn call(&mut self, id: usize, args: &[F], num_outputs: usize) -> Result<Vec<F>, Error> {
        if let Some(hint) = self.hints.get_mut(&id) {
            let mut outputs = vec![F::zero(); num_outputs];
            hint(args, &mut outputs).map(|_| outputs)
        } else {
            panic!("Hint with id {id} not found");
        }
    }
}

#[derive(Default)]
pub struct EmptyHintCaller;

impl EmptyHintCaller {
    pub fn new() -> Self {
        Self
    }
}
pub struct StubHintCaller;

pub trait HintCaller<F: Field>: 'static {
    fn call(&mut self, id: usize, args: &[F], num_outputs: usize) -> Result<Vec<F>, Error>;
    fn custom_gate_type_to_hint_id(&self, gate_type: usize) -> Result<usize, Error> {
        Err(Error::UserError(format!(
            "custom gate type {gate_type} not found"
        )))
    }
}

impl<F: Field + 'static> HintCaller<F> for HintRegistry<F> {
    fn call(&self, id: usize, args: &[F], num_outputs: usize) -> Result<Vec<F>, Error> {
        self.call(id, args, num_outputs)
    }
    fn custom_gate_type_to_hint_id(&self, gate_type: usize) -> Result<usize, Error> {
        self.custom_gate_type_to_hint_id
            .get(&gate_type)
            .cloned()
            .ok_or_else(|| Error::UserError(format!("custom gate type {gate_type} not found")))
    }
}

impl<F: Field> HintCaller<F> for EmptyHintCaller {
    fn call(&self, id: usize, _: &[F], _: usize) -> Result<Vec<F>, Error> {
        Err(Error::UserError(format!("hint with id {id} not found")))
    }
}

impl<F: Field> HintCaller<F> for StubHintCaller {
    fn call(&self, id: usize, args: &[F], num_outputs: usize) -> Result<Vec<F>, Error> {
        Ok(stub_impl(id, &args.to_vec(), num_outputs))
    }
}
