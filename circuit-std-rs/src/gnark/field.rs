use std::collections::HashMap;
use std::rc::Rc;
use crate::gnark::limbs::*;
use crate::gnark::utils::*;
use crate::gnark::emparam::FieldParams;
use crate::gnark::element::*;
use expander_compiler::frontend::extra::*;
use expander_compiler::{circuit::layered::InputType, frontend::*};
use expander_compiler::frontend::builder::*;
use num_bigint::BigInt;
pub struct mul_check<T: FieldParams> {
    a: Element<T>,
    b: Element<T>,
    r: Element<T>,
    k: Element<T>,
    c: Element<T>,
    p: Element<T>,
}
pub struct Field<T: FieldParams> {
    f_params: T,
    max_of: u32,
    n_const: Element<T>,
    nprev_const: Element<T>,
    zero_const: Element<T>,
    one_const: Element<T>,
    short_one_const: Element<T>,
    constrained_limbs: HashMap<usize, ()>,
    //checker: Box<dyn Rangechecker>, we use lookup rangeproof instead
    mul_checks: Vec<mul_check<T>>,
}

impl <T: FieldParams>Field<T> {
    pub fn new<'a, C: Config, B: RootAPI<C>>(native: &'a mut B, f_params: T) -> Self {
        let mut field = Field {
            f_params: f_params,
            max_of: 30 - 2 -  T::bits_per_limb(),
            n_const: Element::<T>::default(),
            nprev_const: Element::<T>::default(),
            zero_const: Element::<T>::default(),
            one_const: Element::<T>::default(),
            short_one_const: Element::<T>::default(),
            constrained_limbs: HashMap::new(),
            mul_checks: Vec::new(),
        };
        field.n_const = value_of::<C, B, T>( native, Box::new(T::modulus()));
        field.nprev_const = value_of::<C, B, T>( native, Box::new(T::modulus()-1));
        field.zero_const = value_of::<C, B, T>( native, Box::new(0));
        field.one_const = value_of::<C, B, T>( native, Box::new(1));
        field.short_one_const = new_internal_element::<T>( vec![native.constant(1);1], 0);
        field
    }
    pub fn max_overflow(&self) -> u64 {
        30 - 2 - 8
    }
    pub fn enforce_width_conditional<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, a: &Element<T>) -> bool{
        let mut did_constrain = false;
       if a.internal {
           return false;
       }
        for i in 0..a.limbs.len() {
            let value_id = get_variable_id(a.limbs[i]);
            if !self.constrained_limbs.contains_key(&value_id) {
                self.constrained_limbs.insert(value_id, ());
            }
             else {
                did_constrain = true;
            }
        }
        self.enforce_width(native, a, true);
        did_constrain
    }
    pub fn enforce_width<'a, C: Config, B: RootAPI<C>>(&self, native: &'a mut B, a: &Element<T>, mod_width: bool) {
        for i in 0..a.limbs.len() {
            let mut limb_nb_bits = T::bits_per_limb() as u64;
            if mod_width && i == a.limbs.len()-1 {
                limb_nb_bits = ((T::modulus().bits() - 1) % T::bits_per_limb() as u64)  + 1;
            }
            //f.checker.Check(a.Limbs[i], limbNbBits)
            let mut inputs = vec![native.constant(limb_nb_bits as u32)];
            inputs.push(a.limbs[i]);
            native.new_hint("myhint.simple_rangecheck_hint", &inputs, 1);
            //logup.RangeProof(f.api, a.Limbs[i], limbNbBits)
        }
    }
    pub fn pack_limbs<'a, C: Config, B: RootAPI<C>>(&self, native: &'a mut B, limbs: Vec<Variable>, strict: bool) -> Element<T> {
        let e = new_internal_element::<T>(limbs, 0);
        self.enforce_width(native, &e, strict);
        e
    }
    pub fn reduce<'a, C: Config, B: RootAPI<C>>(
        &mut self,
        native: &'a mut B,
        a: Element<T>,
        strict: bool,
    ) -> Element<T> {
        self.enforce_width_conditional(native, &a);
        if a.mod_reduced {
            return a;
        }
        if !strict && a.overflow == 0 {
            return a;
        }
        let p = Element::<T>::default();
        let (k, r, c) = self.call_mul_hint(native, &a, &self.one_const, true);
        let mc = mul_check{
            a: a,
            b: self.one_const.clone(),
            c: c,
            k: k,
            r: r.clone(),
            p: p,
        };
        self.mul_checks.push(mc);
        return r
    }
    pub fn mul_pre_cond(&self, a: Element<T>, b: Element<T>) -> u32 {
        let nb_res_limbs = nb_multiplication_res_limbs(a.limbs.len(), b.limbs.len());
        let nb_limbs_overflow = if nb_res_limbs > 0 {
            (nb_res_limbs as f64).log2().ceil() as u32
        } else {
            1
        };
        let next_overflow = T::bits_per_limb() + nb_limbs_overflow + a.overflow + b.overflow;
        next_overflow
    }
    pub fn call_mul_hint<'a, C: Config, B: RootAPI<C>>(
        &self,
        native: &'a mut B,
        a: &Element<T>,
        b: &Element<T>,
        is_mul_mod: bool,
    ) -> (Element<T>, Element<T>, Element<T>) {
        let next_overflow = self.mul_pre_cond(a.clone(), b.clone());
        let next_overflow = if !is_mul_mod {
            a.overflow
        } else {
            next_overflow
        };
        println!("next_overflow: {}", next_overflow);
        let nb_limbs = T::nb_limbs() as usize;
        let nb_bits = T::bits_per_limb() as usize;
        let modbits = T::modulus().bits() as usize;
        let a_limbs_len = a.limbs.len();
        let b_limbs_len = b.limbs.len();
        let nb_quo_limbs = (nb_multiplication_res_limbs(a_limbs_len, b_limbs_len) * nb_bits + next_overflow as usize + 1 - modbits + nb_bits - 1) / nb_bits;
        let nb_rem_limbs = nb_limbs;
        let nb_carry_limbs = std::cmp::max(nb_multiplication_res_limbs(a_limbs_len, b_limbs_len), nb_multiplication_res_limbs(nb_quo_limbs, nb_limbs)) - 1;
        let mut hint_inputs = vec![native.constant(nb_bits as u32), native.constant(nb_limbs as u32), native.constant(a.limbs.len() as u32), native.constant(nb_quo_limbs as u32)];
        let modulus_limbs = self.n_const.limbs.clone();
        hint_inputs.extend(modulus_limbs);
        hint_inputs.extend(a.limbs.clone());
        hint_inputs.extend(b.limbs.clone());
        let ret = native.new_hint("myhint.mulhint", &hint_inputs, nb_quo_limbs + nb_rem_limbs + nb_carry_limbs);
        let quo = self.pack_limbs(native, ret[..nb_quo_limbs].to_vec(), false);
        let rem = if is_mul_mod {
            self.pack_limbs(native, ret[nb_quo_limbs..nb_quo_limbs + nb_rem_limbs].to_vec(), true)
        } else {
            Element::default()
        };
        let carries = new_internal_element::<T>(ret[nb_quo_limbs + nb_rem_limbs..].to_vec(), 0);
        (quo, rem, carries)
    }
    pub fn add<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, a: Element<T>, b: Element<T>) -> Element<T> {
        self.enforce_width_conditional(native, &a.clone());
        self.enforce_width_conditional(native, &b.clone());
        let mut new_a = a.clone();
        let mut new_b = b.clone();
        if a.overflow + 1 > self.max_of {
            new_a = self.reduce(native, a.clone(), false);
        }
        if b.overflow + 1 > self.max_of {   
            new_b = self.reduce(native, b.clone(), false);
        }
        let next_overflow = std::cmp::max(new_a.overflow, new_b.overflow) + 1;
        let nb_limbs = std::cmp::max(new_a.limbs.len(), new_b.limbs.len());
        let mut limbs = vec![native.constant(0); nb_limbs];
        for i in 0..limbs.len() {
            if i < new_a.limbs.len() {
                limbs[i] = native.add(limbs[i], new_a.limbs[i]);
            }
            if i < new_b.limbs.len() {
                limbs[i] = native.add(limbs[i], new_b.limbs[i]);
            }
        }
        let ret = new_internal_element::<T>(limbs, next_overflow);
        ret
    }
}


/*
func (f *Field[T]) Add(a, b *Element[T]) *Element[T] {
	return f.reduceAndOp(f.add, f.addPreCond, a, b)
}

func (f *Field[T]) reduceAndOp(op func(*Element[T], *Element[T], uint) *Element[T], preCond func(*Element[T], *Element[T]) (uint, error), a, b *Element[T]) *Element[T] {
	f.enforceWidthConditional(a)
	f.enforceWidthConditional(b)
	var nextOverflow uint
	var err error
	var target overflowError

	for nextOverflow, err = preCond(a, b); errors.As(err, &target); nextOverflow, err = preCond(a, b) {
		if !target.reduceRight {
			a = f.Reduce(a)
		} else {
			b = f.Reduce(b)
		}
	}
	return op(a, b, nextOverflow)
}

func (f *Field[T]) addPreCond(a, b *Element[T]) (nextOverflow uint, err error) {
	reduceRight := a.overflow < b.overflow
	nextOverflow = max(a.overflow, b.overflow) + 1
	if nextOverflow > f.maxOverflow() {
		err = overflowError{op: "add", nextOverflow: nextOverflow, maxOverflow: f.maxOverflow(), reduceRight: reduceRight}
	}
	return
}

func (f *Field[T]) add(a, b *Element[T], nextOverflow uint) *Element[T] {
	ba, aConst := f.constantValue(a)
	bb, bConst := f.constantValue(b)
	if aConst && bConst {
		ba.Add(ba, bb).Mod(ba, f.fParams.Modulus())
		return newConstElement[T](ba)
	}

	nbLimbs := max(len(a.Limbs), len(b.Limbs))
	limbs := make([]frontend.Variable, nbLimbs)
	for i := range limbs {
		limbs[i] = 0
		if i < len(a.Limbs) {
			limbs[i] = f.api.Add(limbs[i], a.Limbs[i])
		}
		if i < len(b.Limbs) {
			limbs[i] = f.api.Add(limbs[i], b.Limbs[i])
		}
	}
	return f.newInternalElement(limbs, nextOverflow)
}
*/