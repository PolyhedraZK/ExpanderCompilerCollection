use crate::gnark::emparam::FieldParams;
use crate::gnark::emparam::{bls12381_fr, Bls12381Fp};
use crate::gnark::limbs::*;
use expander_compiler::frontend::*;
use num_bigint::BigInt;
use std::any::Any;
use std::cmp::Ordering;
use num_traits::ToPrimitive;
#[derive(Default,Clone)]
pub struct Element<T: FieldParams> {
    pub limbs: Vec<Variable>,
    pub overflow: u32,
    pub internal: bool,
    pub mod_reduced: bool,
    pub is_evaluated: bool,
    pub evaluation: Variable,
    pub _marker: std::marker::PhantomData<T>,
}

impl <T: FieldParams>Element<T> {
    pub fn new(limbs: Vec<Variable>, overflow: u32, internal: bool, mod_reduced: bool, is_evaluated: bool, evaluation: Variable) -> Self {
        Self {
            limbs,
            overflow,
            internal,
            mod_reduced,
            is_evaluated,
            evaluation,
            _marker: std::marker::PhantomData,
        }
    }
    pub fn default() -> Self {
        Self {
            limbs: Vec::new(),
            overflow: 0,
            internal: false,
            mod_reduced: false,
            is_evaluated: false,
            evaluation: Variable::default(),
            _marker: std::marker::PhantomData,
        }
    }
    pub fn clone(&self) -> Self {
        Self {
            limbs: self.limbs.clone(),
            overflow: self.overflow,
            internal: self.internal,
            mod_reduced: self.mod_reduced,
            is_evaluated: self.is_evaluated,
            evaluation: self.evaluation,
            _marker: std::marker::PhantomData,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.limbs.is_empty()
    }
}
pub fn value_of<'a, C: Config, B: RootAPI<C>, T: FieldParams>(api: &'a mut B, constant: Box<dyn Any>) -> Element<T> {
    let r:Element<T> = new_const_element::<C,B,T>(api, constant);
    r
}
pub fn new_const_element<C: Config, B: RootAPI<C>, T: FieldParams>(api: &mut B, v: Box<dyn Any>) -> Element<T> {
    let fp = T::modulus();
    // convert to big.Int
    let mut b_value = from_interface(v);
    // mod reduce
    if fp.cmp(&b_value) != Ordering::Equal {
        b_value = b_value % fp;
    }

    // decompose into limbs
    // TODO @gbotrel use big.Int pool here
    let mut blimbs = vec![BigInt::default(); T::nb_limbs() as usize];
    let mut limbs = vec![Variable::default(); blimbs.len()];
    if let Err(err) = decompose(&b_value, T::bits_per_limb(), &mut blimbs) {
        panic!("decompose value: {}", err);
    }
    // assign limb values
    for i in 0..limbs.len() {
        limbs[i] = api.constant(blimbs[i].to_u64().unwrap() as u32);
    }
    Element::new(limbs, 0, true, false, false, Variable::default())
}
pub fn new_internal_element<T: FieldParams>(limbs: Vec<Variable>, overflow: u32) -> Element<T> {
    Element::new(limbs, overflow, true, false, false, Variable::default())
}
pub fn copy<T: FieldParams>(e: &Element<T>) -> Element<T> {
    let mut r = Element::new(Vec::new(), 0, false, false, false, Variable::default());
    r.limbs = e.limbs.clone();
    r.overflow = e.overflow;
    r.internal = e.internal;
    r.mod_reduced = e.mod_reduced;
    r
}
pub fn from_interface(input: Box<dyn Any>) -> BigInt {
    let mut r = BigInt::from(0);

    if let Some(v) = input.downcast_ref::<BigInt>() {
        r = v.clone();
    } else if let Some(v) = input.downcast_ref::<u8>() {
        r = BigInt::from(*v);
    } else if let Some(v) = input.downcast_ref::<u16>() {
        r = BigInt::from(*v);
    } else if let Some(v) = input.downcast_ref::<u32>() {
        r = BigInt::from(*v);
    } else if let Some(v) = input.downcast_ref::<u64>() {
        r = BigInt::from(*v);
    } else if let Some(v) = input.downcast_ref::<usize>() {
        r = BigInt::from(*v as u64);
    } else if let Some(v) = input.downcast_ref::<i8>() {
        r = BigInt::from(*v);
    } else if let Some(v) = input.downcast_ref::<i16>() {
        r = BigInt::from(*v);
    } else if let Some(v) = input.downcast_ref::<i32>() {
        r = BigInt::from(*v);
    } else if let Some(v) = input.downcast_ref::<i64>() {
        r = BigInt::from(*v);
    } else if let Some(v) = input.downcast_ref::<isize>() {
        r = BigInt::from(*v as i64);
    } else if let Some(v) = input.downcast_ref::<String>() {
        r = BigInt::parse_bytes(v.as_bytes(), 10).unwrap_or_else(|| {
            panic!("unable to set BigInt from string: {}", v);
        });
    } else if let Some(v) = input.downcast_ref::<Vec<u8>>() {
        r = BigInt::from_bytes_be(num_bigint::Sign::Plus, v);
    } else {
        panic!(
            "value to BigInt not supported"
        );
    }

    r
}



#[test]
fn test_from_interface() {
    let v = 1111111u32;
    let r = from_interface(Box::new(v));
    assert_eq!(r, BigInt::from(1111111u32));
    let v = 22222222222222u64;
    let r = from_interface(Box::new(v));
    assert_eq!(r, BigInt::from(22222222222222u64));
    let v = 333333usize;
    let r = from_interface(Box::new(v));
    assert_eq!(r, BigInt::from(333333usize as u64));
    let v = 444444i32;
    let r = from_interface(Box::new(v));
    assert_eq!(r, BigInt::from(444444i32));
    let v = 555555555555555i64;
    let r = from_interface(Box::new(v));
    assert_eq!(r, BigInt::from(555555555555555i64));
    let v = 666isize;
    let r = from_interface(Box::new(v));
    assert_eq!(r, BigInt::from(666isize as i64));
    let v = "77777777777777777".to_string();
    let r = from_interface(Box::new(v));
    assert_eq!(r, BigInt::from(77777777777777777u64));
    let v = vec![7u8;4];
    let r = from_interface(Box::new(v));
    assert_eq!(r, BigInt::from(0x07070707u32));
}

declare_circuit!(VALUECircuit {
	target: [[Variable;48];8],
});
impl Define<M31Config> for VALUECircuit<Variable> {
    fn define(&self, builder: &mut API<M31Config>) {
        let mut targets = Vec::<Variable>::new();
        let v1 = 1111111u32;
        let v2 = 22222222222222u64;
        let v3 = 333333usize;
        let v4 = 444444i32;
        let v5 = 555555555555555i64;
        let v6 = 666isize;
        let v7 = "77777777777777777".to_string();
        let v8 = vec![8u8;4];
        let mut rs = vec![];
        let r1 = value_of::<M31Config,_,Bls12381Fp>(builder, Box::new(v1));
        let r2 = value_of::<M31Config,_,Bls12381Fp>(builder, Box::new(v2));
        let r3 = value_of::<M31Config,_,Bls12381Fp>(builder, Box::new(v3));
        let r4 = value_of::<M31Config,_,Bls12381Fp>(builder, Box::new(v4));
        let r5 = value_of::<M31Config,_,Bls12381Fp>(builder, Box::new(v5));
        let r6 = value_of::<M31Config,_,Bls12381Fp>(builder, Box::new(v6));
        let r7 = value_of::<M31Config,_,Bls12381Fp>(builder, Box::new(v7));
        let r8 = value_of::<M31Config,_,Bls12381Fp>(builder, Box::new(v8));
        rs = vec![r1, r2, r3, r4, r5, r6, r7, r8];
        for i in 0..rs.len() {
            for j in 0..rs[i].limbs.len() {
                let left = builder.constant(rs[i].limbs[j]);
                let right = builder.constant(self.target[i][j]);
                builder.assert_is_equal(rs[i].limbs[j], self.target[i][j]);
            }
        }
    }  
}

#[test]
fn test_value() {
    let values:Vec<u64> = vec![1111111, 22222222222222, 333333, 444444, 555555555555555, 666, 77777777777777777, 0x08080808];
    let values_u8: Vec<Vec<u8>> = values.iter().map(|v| v.to_le_bytes().to_vec()).collect();
    let compile_result = compile(&VALUECircuit::default()).unwrap();
    let mut assignment = VALUECircuit::<M31>::default();
    for i in 0..values_u8.len() {
        for j in 0..values_u8[i].len() {
            assignment.target[i][j] = M31::from(values_u8[i][j] as u32);
        }
    }
    let witness = compile_result
        .witness_solver
        .solve_witness(&assignment)
        .unwrap();
    let output = compile_result.layered_circuit.run(&witness);
    assert_eq!(output, vec![true]);
}