use std::hash::{DefaultHasher, Hash, Hasher};

use rand::RngCore;

use crate::{
    field::{Field, U256},
    utils::error::Error,
};

pub enum BuiltinHintIds {
    Identity = 0xccc000000000,
    Div,
    Eq,
    NotEq,
    BoolOr,
    BoolAnd,
    BitOr,
    BitAnd,
    BitXor,
    Select,
    Pow,
    IntDiv,
    Mod,
    ShiftL,
    ShiftR,
    LesserEq,
    GreaterEq,
    Lesser,
    Greater,
}

impl BuiltinHintIds {
    pub fn from_usize(id: usize) -> Option<BuiltinHintIds> {
        if id < (BuiltinHintIds::Identity as usize) {
            return None;
        }
        if id > (BuiltinHintIds::Identity as usize + 100) {
            return None;
        }
        match id {
            x if x == BuiltinHintIds::Identity as usize => Some(BuiltinHintIds::Identity),
            x if x == BuiltinHintIds::Div as usize => Some(BuiltinHintIds::Div),
            x if x == BuiltinHintIds::Eq as usize => Some(BuiltinHintIds::Eq),
            x if x == BuiltinHintIds::NotEq as usize => Some(BuiltinHintIds::NotEq),
            x if x == BuiltinHintIds::BoolOr as usize => Some(BuiltinHintIds::BoolOr),
            x if x == BuiltinHintIds::BoolAnd as usize => Some(BuiltinHintIds::BoolAnd),
            x if x == BuiltinHintIds::BitOr as usize => Some(BuiltinHintIds::BitOr),
            x if x == BuiltinHintIds::BitAnd as usize => Some(BuiltinHintIds::BitAnd),
            x if x == BuiltinHintIds::BitXor as usize => Some(BuiltinHintIds::BitXor),
            x if x == BuiltinHintIds::Select as usize => Some(BuiltinHintIds::Select),
            x if x == BuiltinHintIds::Pow as usize => Some(BuiltinHintIds::Pow),
            x if x == BuiltinHintIds::IntDiv as usize => Some(BuiltinHintIds::IntDiv),
            x if x == BuiltinHintIds::Mod as usize => Some(BuiltinHintIds::Mod),
            x if x == BuiltinHintIds::ShiftL as usize => Some(BuiltinHintIds::ShiftL),
            x if x == BuiltinHintIds::ShiftR as usize => Some(BuiltinHintIds::ShiftR),
            x if x == BuiltinHintIds::LesserEq as usize => Some(BuiltinHintIds::LesserEq),
            x if x == BuiltinHintIds::GreaterEq as usize => Some(BuiltinHintIds::GreaterEq),
            x if x == BuiltinHintIds::Lesser as usize => Some(BuiltinHintIds::Lesser),
            x if x == BuiltinHintIds::Greater as usize => Some(BuiltinHintIds::Greater),
            _ => None,
        }
    }
}

fn stub_impl_general<F: Field>(hint_id: usize, inputs: &Vec<F>, num_outputs: usize) -> Vec<F> {
    let mut hasher = DefaultHasher::new();
    hint_id.hash(&mut hasher);
    inputs.hash(&mut hasher);
    let mut outputs = Vec::with_capacity(num_outputs);
    for _ in 0..num_outputs {
        let t = hasher.finish();
        outputs.push(F::from(t as u32));
        t.hash(&mut hasher);
    }
    outputs
}

fn validate_builtin_hint(
    hint_id: BuiltinHintIds,
    num_inputs: usize,
    num_outputs: usize,
) -> Result<(), Error> {
    match hint_id {
        BuiltinHintIds::Identity => {
            if num_inputs != num_outputs {
                return Err(Error::InternalError(
                    "identity hint requires exactly the same number of inputs and outputs"
                        .to_string(),
                ));
            }
            if num_inputs == 0 {
                return Err(Error::InternalError(
                    "identity hint requires at least 1 input".to_string(),
                ));
            }
        }
        BuiltinHintIds::Div
        | BuiltinHintIds::Eq
        | BuiltinHintIds::NotEq
        | BuiltinHintIds::BoolOr
        | BuiltinHintIds::BoolAnd
        | BuiltinHintIds::BitOr
        | BuiltinHintIds::BitAnd
        | BuiltinHintIds::BitXor
        | BuiltinHintIds::Pow
        | BuiltinHintIds::IntDiv
        | BuiltinHintIds::Mod
        | BuiltinHintIds::ShiftL
        | BuiltinHintIds::ShiftR
        | BuiltinHintIds::LesserEq
        | BuiltinHintIds::GreaterEq
        | BuiltinHintIds::Lesser
        | BuiltinHintIds::Greater => {
            if num_inputs != 2 {
                return Err(Error::InternalError(
                    "binary op requires exactly 2 inputs".to_string(),
                ));
            }
            if num_outputs != 1 {
                return Err(Error::InternalError(
                    "binary op requires exactly 1 output".to_string(),
                ));
            }
        }
        BuiltinHintIds::Select => {
            if num_inputs != 3 {
                return Err(Error::InternalError(
                    "select requires exactly 3 inputs".to_string(),
                ));
            }
            if num_outputs != 1 {
                return Err(Error::InternalError(
                    "select requires exactly 1 output".to_string(),
                ));
            }
        }
    }
    Ok(())
}

pub fn validate_hint(hint_id: usize, num_inputs: usize, num_outputs: usize) -> Result<(), Error> {
    match BuiltinHintIds::from_usize(hint_id) {
        Some(hint_id) => validate_builtin_hint(hint_id, num_inputs, num_outputs),
        None => {
            if num_outputs == 0 {
                return Err(Error::InternalError(
                    "custom hint requires at least 1 output".to_string(),
                ));
            }
            if num_inputs == 0 {
                return Err(Error::InternalError(
                    "custom hint requires at least 1 input".to_string(),
                ));
            }
            Ok(())
        }
    }
}

fn impl_builtin_hint<F: Field>(
    hint_id: BuiltinHintIds,
    inputs: &Vec<F>,
    num_outputs: usize,
) -> Vec<F> {
    match hint_id {
        BuiltinHintIds::Identity => {
            let mut outputs = Vec::with_capacity(num_outputs);
            for i in 0..num_outputs {
                outputs.push(inputs[i]);
            }
            outputs
        }
        BuiltinHintIds::Div => binop_hint(inputs, |x, y| match y.inv() {
            Some(inv) => x * inv,
            None => F::zero(),
        }),
        BuiltinHintIds::Eq => binop_hint(inputs, |x, y| F::from((x == y) as u32)),
        BuiltinHintIds::NotEq => binop_hint(inputs, |x, y| F::from((x != y) as u32)),
        BuiltinHintIds::BoolOr => binop_hint(inputs, |x, y| {
            F::from((!x.is_zero() || !y.is_zero()) as u32)
        }),
        BuiltinHintIds::BoolAnd => binop_hint(inputs, |x, y| {
            F::from((!x.is_zero() && !y.is_zero()) as u32)
        }),
        BuiltinHintIds::BitOr => binop_hint_on_u256(inputs, |x, y| x | y),
        BuiltinHintIds::BitAnd => binop_hint_on_u256(inputs, |x, y| x & y),
        BuiltinHintIds::BitXor => binop_hint_on_u256(inputs, |x, y| x ^ y),
        BuiltinHintIds::Select => {
            let mut outputs = Vec::with_capacity(num_outputs);
            outputs.push(if !inputs[0].is_zero() {
                inputs[1]
            } else {
                inputs[2]
            });
            outputs
        }
        BuiltinHintIds::Pow => binop_hint(inputs, |x, y| {
            let mut t = x;
            let mut res = F::one();
            let mut y: U256 = y.into();
            while !y.is_zero() {
                if y.bit(0) {
                    res = res * t;
                }
                y = y >> 1;
                t = t * t;
            }
            res
        }),
        BuiltinHintIds::IntDiv => {
            binop_hint_on_u256(
                inputs,
                |x, y| if y.is_zero() { U256::zero() } else { x / y },
            )
        }
        BuiltinHintIds::Mod => {
            binop_hint_on_u256(
                inputs,
                |x, y| if y.is_zero() { U256::zero() } else { x % y },
            )
        }
        BuiltinHintIds::ShiftL => binop_hint_on_u256(inputs, |x, y| circom_shift_l_impl::<F>(x, y)),
        BuiltinHintIds::ShiftR => binop_hint_on_u256(inputs, |x, y| circom_shift_r_impl::<F>(x, y)),
        BuiltinHintIds::LesserEq => binop_hint(inputs, |x, y| F::from((x <= y) as u32)),
        BuiltinHintIds::GreaterEq => binop_hint(inputs, |x, y| F::from((x >= y) as u32)),
        BuiltinHintIds::Lesser => binop_hint(inputs, |x, y| F::from((x < y) as u32)),
        BuiltinHintIds::Greater => binop_hint(inputs, |x, y| F::from((x > y) as u32)),
    }
}

fn binop_hint<F: Field, G: Fn(F, F) -> F>(inputs: &Vec<F>, f: G) -> Vec<F> {
    vec![f(inputs[0], inputs[1])]
}

fn binop_hint_on_u256<F: Field, T: Into<F>, G: Fn(U256, U256) -> T>(
    inputs: &Vec<F>,
    f: G,
) -> Vec<F> {
    let x_u256: U256 = inputs[0].into();
    let y_u256: U256 = inputs[1].into();
    let z_u256 = f(x_u256, y_u256);
    vec![z_u256.into()]
}

pub fn stub_impl<F: Field>(hint_id: usize, inputs: &Vec<F>, num_outputs: usize) -> Vec<F> {
    match BuiltinHintIds::from_usize(hint_id) {
        Some(hint_id) => impl_builtin_hint(hint_id, inputs, num_outputs),
        None => stub_impl_general(hint_id, inputs, num_outputs),
    }
}

pub fn random_builtin(mut rand: impl RngCore) -> (usize, usize, usize) {
    loop {
        let hint_id = (rand.next_u64() as usize % 100) + (BuiltinHintIds::Identity as usize);
        if let Some(hint_id) = BuiltinHintIds::from_usize(hint_id) {
            match hint_id {
                BuiltinHintIds::Identity => {
                    let num_inputs = (rand.next_u64() % 10) as usize + 1;
                    let num_outputs = num_inputs;
                    return (hint_id as usize, num_inputs, num_outputs);
                }
                BuiltinHintIds::Div
                | BuiltinHintIds::Eq
                | BuiltinHintIds::NotEq
                | BuiltinHintIds::BoolOr
                | BuiltinHintIds::BoolAnd
                | BuiltinHintIds::BitOr
                | BuiltinHintIds::BitAnd
                | BuiltinHintIds::BitXor
                | BuiltinHintIds::Pow
                | BuiltinHintIds::IntDiv
                | BuiltinHintIds::Mod
                | BuiltinHintIds::ShiftL
                | BuiltinHintIds::ShiftR
                | BuiltinHintIds::LesserEq
                | BuiltinHintIds::GreaterEq
                | BuiltinHintIds::Lesser
                | BuiltinHintIds::Greater => {
                    return (hint_id as usize, 2, 1);
                }
                BuiltinHintIds::Select => {
                    return (hint_id as usize, 3, 1);
                }
            }
        }
    }
}

pub fn circom_shift_l_impl<F: Field>(x: U256, k: U256) -> U256 {
    let top = F::modulus() / 2;
    if k <= top {
        let shift = if (k >> 64).is_zero() {
            k.as_u64() as usize
        } else {
            F::modulus().bits()
        };
        let value = x << shift;
        let mask = U256::from(1) << F::modulus().bits();
        let mask = mask - 1;
        let value = value & mask;
        value
    } else {
        circom_shift_r_impl::<F>(x, F::modulus() - k)
    }
}

pub fn circom_shift_r_impl<F: Field>(x: U256, k: U256) -> U256 {
    let top = F::modulus() / 2;
    if k <= top {
        let shift = if (k >> 64).is_zero() {
            k.as_u64() as usize
        } else {
            F::modulus().bits()
        };
        let value = x >> shift;
        value
    } else {
        circom_shift_l_impl::<F>(x, F::modulus() - k)
    }
}