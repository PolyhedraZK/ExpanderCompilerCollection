use std::collections::HashMap;
use std::rc::Rc;
// use crate::gnark::emulated::field_bls12381::e2::print_e2;
// use crate::gnark::emulated::field_bls12381::e2::print_element;
use crate::gnark::limbs::*;
use crate::gnark::utils::*;
use crate::gnark::emparam::FieldParams;
use crate::gnark::element::*;
use crate::logup::LogUpRangeProofTable;
use crate::utils::simple_select;
use expander_compiler::frontend::extra::*;
use expander_compiler::{circuit::layered::InputType, frontend::*};
use expander_compiler::frontend::builder::*;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use num_traits::Zero;
use num_traits::Signed; 

pub struct mul_check<T: FieldParams> {
    a: Element<T>,
    b: Element<T>,
    r: Element<T>,
    k: Element<T>,
    c: Element<T>,
    p: Element<T>,
}
impl <T: FieldParams>mul_check<T> {
    pub fn eval_round1<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, at: Vec<Variable>) {
        // println!("c");
        // print_element(native, &self.c);
        // println!("r");
        // print_element(native, &self.r);
        // println!("k");
        // print_element(native, &self.k);
        self.c = eval_with_challenge(native, self.c.clone(), at.clone());
        self.r = eval_with_challenge(native,self.r.clone(), at.clone());
        self.k = eval_with_challenge(native, self.k.clone(), at.clone());
        if !self.p.is_empty() {
            self.p = eval_with_challenge(native,self.p.clone(), at.clone());
        }
        // println!("c:{:?}", native.value_of(self.c.evaluation));
        // println!("r:{:?}", native.value_of(self.r.evaluation));
        // println!("k:{:?}", native.value_of(self.k.evaluation));
        // println!("p:{:?}", native.value_of(self.p.evaluation));

    }
    pub fn eval_round2<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, at: Vec<Variable>) {
        // println!("a");
        // print_element(native, &self.a);
        // println!("b");
        // print_element(native, &self.b);
        self.a = eval_with_challenge(native, self.a.clone(), at.clone());
        self.b = eval_with_challenge(native, self.b.clone(), at.clone());
        // println!("a:{:?}", native.value_of(self.a.evaluation));
        // println!("b:{:?}", native.value_of(self.b.evaluation));
    }
    pub fn check<'a, C: Config, B: RootAPI<C>>(&self, native: &'a mut B, pval: Variable, ccoef: Variable) {
        let mut new_peval = pval;
        if !self.p.is_empty() {
            new_peval = self.p.evaluation
        };
        // println!("ls_a:{:?}", native.value_of(self.a.evaluation));
        // println!("ls_b:{:?}", native.value_of(self.b.evaluation));
        let ls = native.mul(self.a.evaluation, self.b.evaluation);
        let rs_tmp1 = native.mul(new_peval, self.k.evaluation);
        // println!("rs_tmp1:{:?}", native.value_of(rs_tmp1));
        let rs_tmp2 = native.mul(self.c.evaluation, ccoef);
        // println!("rs_tmp2:{:?}", native.value_of(rs_tmp2));
        let rs_tmp3 = native.add(self.r.evaluation, rs_tmp1);
        // println!("rs_tmp3:{:?}", native.value_of(rs_tmp3));
        let rs = native.add(rs_tmp3, rs_tmp2);
        // println!("ls:{:?}", native.value_of(ls));
        // println!("rs:{:?}", native.value_of(rs));
        native.assert_is_equal(ls, rs);
    }
    pub fn clean_evaluations(&mut self) {
        self.a.evaluation = Variable::default();
        self.a.is_evaluated = false;
        self.b.evaluation = Variable::default();
        self.b.is_evaluated = false;
        self.r.evaluation = Variable::default();
        self.r.is_evaluated = false;
        self.k.evaluation = Variable::default();
        self.k.is_evaluated = false;
        self.c.evaluation = Variable::default();
        self.c.is_evaluated = false;
        self.p.evaluation = Variable::default();
        self.p.is_evaluated = false;
    }
}
pub struct Field<T: FieldParams> {
    f_params: T,
    max_of: u32,
    n_const: Element<T>,
    nprev_const: Element<T>,
    pub zero_const: Element<T>,
    pub one_const: Element<T>,
    short_one_const: Element<T>,
    constrained_limbs: HashMap<usize, ()>,
    pub table: LogUpRangeProofTable,
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
            table: LogUpRangeProofTable::new(8),
            mul_checks: Vec::new(),
        };
        field.n_const = value_of::<C, B, T>( native, Box::new(T::modulus()));
        field.nprev_const = value_of::<C, B, T>( native, Box::new(T::modulus()-1));
        field.zero_const = value_of::<C, B, T>( native, Box::new(0));
        field.one_const = value_of::<C, B, T>( native, Box::new(1));
        field.short_one_const = new_internal_element::<T>( vec![native.constant(1);1], 0);
        field.table.initial(native);
        field
    }
    pub fn max_overflow(&self) -> u64 {
        30 - 2 - 8
    }
    pub fn is_zero<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, a: &Element<T>) -> Variable {
        let ca = self.reduce(native, a, false);
        let mut res0 = native.constant(1);
        let total_overflow = ca.limbs.len() as i32 - 1;
        if total_overflow > self.max_overflow() as i32 {
            res0 = native.is_zero(ca.limbs[0]);
            for i in 1..ca.limbs.len() {
                let tmp = native.is_zero(ca.limbs[i]);
                res0 = native.mul(res0, tmp);
            }
        } else {
            let mut limb_sum = ca.limbs[0];
            for i in 1..ca.limbs.len() {
                limb_sum = native.add(limb_sum, ca.limbs[i]);
            }
            res0 = native.is_zero(limb_sum);
        }
        res0
    }
    pub fn select<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, selector: Variable, a: &Element<T>, b: &Element<T>) -> Element<T> {
        self.enforce_width_conditional(native, &a.clone());
        self.enforce_width_conditional(native, &b.clone());
        let overflow = std::cmp::max(a.overflow, b.overflow);
        let nb_limbs = std::cmp::max(a.limbs.len(), b.limbs.len());
        let mut limbs = vec![native.constant(0); nb_limbs];
        let mut normalize = |limbs: Vec<Variable>| -> Vec<Variable> {
            if limbs.len() < nb_limbs {
                let mut tail = vec![native.constant(0); nb_limbs - limbs.len()];
                for i in 0..tail.len() {
                    tail[i] = native.constant(0);
                }
                return limbs.iter().chain(tail.iter()).cloned().collect();
            }
            limbs
        };
        let a_norm_limbs = normalize(a.limbs.clone());
        let b_norm_limbs = normalize(b.limbs.clone());
        for i in 0..limbs.len() {
            limbs[i] = simple_select(native, selector, a_norm_limbs[i], b_norm_limbs[i]);
        }
        let e = new_internal_element::<T>(limbs, overflow);
        e
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
    pub fn enforce_width<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, a: &Element<T>, mod_width: bool) {
        for i in 0..a.limbs.len() {
            let mut limb_nb_bits = T::bits_per_limb() as u64;
            if mod_width && i == a.limbs.len()-1 {
                limb_nb_bits = ((T::modulus().bits() - 1) % T::bits_per_limb() as u64)  + 1;
            }
            //range check
            if limb_nb_bits > 8 {
                self.table.rangeproof(native, a.limbs[i], limb_nb_bits as usize);
            } else {
                self.table.rangeproof_onechunk(native, a.limbs[i], limb_nb_bits as usize);
            }
        }
    }
    pub fn wrap_hint<'a, C: Config, B: RootAPI<C>>(&self, native: &'a mut B, nonnative_inputs: Vec<Element<T>>) -> Vec<Variable> {
        let mut res = vec![native.constant(T::bits_per_limb() as u32), native.constant(T::nb_limbs() as u32)];
        res.extend(self.n_const.limbs.clone());
        res.push(native.constant(nonnative_inputs.len() as u32));
        for i in 0..nonnative_inputs.len() {
            res.push(native.constant(nonnative_inputs[i].limbs.len() as u32));
            res.extend(nonnative_inputs[i].limbs.clone());
        }
        res
    }
    pub fn new_hint<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, hf_name: &str, nb_outputs: usize, inputs: Vec<Element<T>>) -> Vec<Element<T>> {
        let native_inputs = self.wrap_hint(native, inputs);
        let nb_native_outputs = T::nb_limbs() as usize * nb_outputs;
        let native_outputs = native.new_hint(hf_name,  &native_inputs, nb_native_outputs);
        let mut outputs = vec![];
        for i in 0..nb_outputs {
            let tmp_output = self.pack_limbs(native, native_outputs[i*T::nb_limbs() as usize..(i+1)*T::nb_limbs() as usize].to_vec(), true);
            outputs.push(tmp_output);
        }
        outputs
    }
    pub fn pack_limbs<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, limbs: Vec<Variable>, strict: bool) -> Element<T> {
        let e = new_internal_element::<T>(limbs, 0);
        self.enforce_width(native, &e, strict);
        e
    }
    pub fn reduce<'a, C: Config, B: RootAPI<C>>(
        &mut self,
        native: &'a mut B,
        a: &Element<T>,
        strict: bool,
    ) -> Element<T> {
        self.enforce_width_conditional(native, a);
        if a.mod_reduced {
            return a.clone();
        }
        if !strict && a.overflow == 0 {
            return a.clone();
        }
        let p = Element::<T>::default();
        let one = self.one_const.clone();
        let res = self.mul_mod(native, a, &one, 0, &p).clone();
        res
    }
    pub fn mul_mod<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, a: &Element<T>, b: &Element<T>, _: usize, p: &Element<T>) -> Element<T> {
        self.enforce_width_conditional(native, &a);
        self.enforce_width_conditional(native, &b);
        // self.enforce_width_conditional(native, &p); //not needed
        let (k, r, c) = self.call_mul_hint(native, &a, &b, true);
        // println!("a,b after call_mul_hint");
        // print_element(native, &a);
        // print_element(native, &b);
        // print_element(native, &r);
        // print_element(native, &c);
        let mc = mul_check{
            a: a.clone(),
            b: b.clone(),
            c: c,
            k: k,
            r: r.clone(),
            p: p.clone(),
        };
        self.mul_checks.push(mc);
        return r
    }
    pub fn mul_pre_cond(&self, a: &Element<T>, b: &Element<T>) -> u32 {
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
        &mut self,
        native: &'a mut B,
        a: &Element<T>,
        b: &Element<T>,
        is_mul_mod: bool,
    ) -> (Element<T>, Element<T>, Element<T>) {
        let next_overflow = self.mul_pre_cond(a, b);
        let next_overflow = if !is_mul_mod {
            a.overflow
        } else {
            next_overflow
        };
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
    pub fn check_zero<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, a: Element<T>, p: Option<Element<T>>) {
        self.enforce_width_conditional(native, &a.clone());
        let b = self.short_one_const.clone();
        // // println!("a,b after call_mul_hint");
        // print_element(native, &a);
        // print_element(native, &b);
        let (k, r, c) = self.call_mul_hint(native, &a, &b, false);
        let mc = mul_check{
            a: a,
            b: b,
            c: c,
            k: k,
            r: r.clone(),
            p: p.unwrap_or(Element::<T>::default()),
        };
        self.mul_checks.push(mc);
    }
    pub fn assert_isequal<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, a: &Element<T>, b: &Element<T>) {
        self.enforce_width_conditional(native, a);
        self.enforce_width_conditional(native, b);
        let diff = self.sub(native, b, a);
        self.check_zero(native, diff, None);
    }
    pub fn add<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, a: &Element<T>, b: &Element<T>) -> Element<T> {
        self.enforce_width_conditional(native, &a.clone());
        self.enforce_width_conditional(native, &b.clone());
        let mut new_a = a.clone();
        let mut new_b = b.clone();
        if a.overflow + 1 > self.max_of {
            new_a = self.reduce(native, a, false);
        }
        if b.overflow + 1 > self.max_of {   
            new_b = self.reduce(native, b, false);
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
    pub fn sub<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, a: &Element<T>, b: &Element<T>) -> Element<T> {
        self.enforce_width_conditional(native, &a.clone());
        self.enforce_width_conditional(native, &b.clone());
        let mut new_a = a.clone();
        let mut new_b = b.clone();
        if a.overflow + 1 > self.max_of {
            new_a = self.reduce(native, a, false);
        }
        if b.overflow + 2 > self.max_of {   
            new_b = self.reduce(native, b, false);
        }
        let next_overflow = std::cmp::max(new_a.overflow, new_b.overflow+1) + 1;
        let nb_limbs = std::cmp::max(new_a.limbs.len(), new_b.limbs.len());
        let pad_limbs = sub_padding(&T::modulus(), T::bits_per_limb(), new_b.overflow, nb_limbs as u32);
        let mut limbs = vec![native.constant(0); nb_limbs];
        for i in 0..limbs.len() {
            limbs[i] = native.constant(pad_limbs[i].to_u64().unwrap() as u32);
            if i < new_a.limbs.len() {
                limbs[i] = native.add(limbs[i], new_a.limbs[i]);
            }
            if i < new_b.limbs.len() {
                limbs[i] = native.sub(limbs[i], new_b.limbs[i]);
            }
        }
        let ret = new_internal_element::<T>(limbs, next_overflow);
        ret
    }
    pub fn neg<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, a: &Element<T>) -> Element<T> {
        let zero = self.zero_const.clone();
        self.sub(native, &zero, a)
    }
    pub fn mul<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, a: &Element<T>, b: &Element<T>) -> Element<T> {
        self.enforce_width_conditional(native, a);
        self.enforce_width_conditional(native, b);

        //calculate a*b's overflow and reduce if necessary
        let mut next_overflow = self.mul_pre_cond(a, b);
        let mut new_a = a.clone();
        let mut new_b = b.clone();
        if next_overflow > self.max_of {
            if a.overflow < b.overflow {
                new_b = self.reduce(native, b, false);
            } else {
                new_a = self.reduce(native, a, false);
            }
        }
        next_overflow = self.mul_pre_cond(&new_a, &new_b);
        if next_overflow > self.max_of {
            if new_a.overflow < new_b.overflow {
                new_b = self.reduce(native, &new_b, false);
            } else {
                new_a = self.reduce(native, &new_a, false);
            }
        }

        //calculate a*b
        return self.mul_mod(native, &new_a, &new_b, 0, &Element::<T>::default());
    }
    pub fn div<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, a: &Element<T>, b: &Element<T>) -> Element<T> {
        self.enforce_width_conditional(native, a);
        self.enforce_width_conditional(native, b);
        //calculate a/b's overflow and reduce if necessary
        let zero_element = self.zero_const.clone();
        let mut mul_of = self.mul_pre_cond(&zero_element, b);
        let mut new_a = a.clone();
        let mut new_b = b.clone();
        if mul_of > self.max_of {
            new_b = self.reduce(native, &new_b, false);
            mul_of = 0;
        }
        if new_a.overflow + 1 > self.max_of {
            new_a = self.reduce(native, &new_a, false);
        }
        if mul_of + 2 > self.max_of {   
            new_b = self.reduce(native, &new_b, false);
        }
        let next_overflow = std::cmp::max(new_a.overflow, new_b.overflow+1) + 1;

        //calculate a/b
        let div = self.compute_division_hint(native, a.limbs.clone(), b.limbs.clone());
        let e = self.pack_limbs(native, div, true);
        let res = self.mul(native, &e, &new_b);
        self.assert_isequal(native, &res, &new_a);
        e
    }
    /*
    mulOf, err := f.mulPreCond(a, &Element[T]{Limbs: make([]frontend.Variable, f.fParams.NbLimbs()), overflow: 0}) // order is important, we want that reduce left side
	if err != nil {
		return mulOf, err
	}
	return f.subPreCond(&Element[T]{overflow: 0}, &Element[T]{overflow: mulOf})
     */
    pub fn inverse<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, b: &Element<T>) -> Element<T> {
        self.enforce_width_conditional(native, b);
        //calculate 1/b's overflow and reduce if necessary
        let zero_element = self.zero_const.clone();
        let mut mul_of = self.mul_pre_cond(&zero_element, b);
        let mut new_b = b.clone();
        if mul_of > self.max_of {
            new_b = self.reduce(native, &new_b, false);
            mul_of = 0;
        }
        if mul_of + 2 > self.max_of {   
            new_b = self.reduce(native, &new_b, false);
        }
        // let next_overflow = std::cmp::max(new_a.overflow, new_b.overflow+1) + 1;

        //calculate 1/b
        let inv = self.compute_inverse_hint(native, b.limbs.clone());
        let e = self.pack_limbs(native, inv, true);
        let res = self.mul(native, &e, &new_b);
        let one = self.one_const.clone();
        self.assert_isequal(native, &res, &one);
        e
    }
    pub fn compute_inverse_hint<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, in_limbs: Vec<Variable>) -> Vec<Variable> {
        let mut hint_inputs = vec![native.constant(T::bits_per_limb() as u32), native.constant(T::nb_limbs() as u32)];
        let modulus_limbs = self.n_const.limbs.clone();
        hint_inputs.extend(modulus_limbs);
        hint_inputs.extend(in_limbs);
        native.new_hint("myhint.invhint", &hint_inputs, T::nb_limbs() as usize)
    }
    pub fn compute_division_hint<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, nom_limbs: Vec<Variable>, denom_limbs: Vec<Variable>) -> Vec<Variable> {
        let mut hint_inputs = vec![native.constant(T::bits_per_limb() as u32), native.constant(T::nb_limbs() as u32), native.constant(denom_limbs.len() as u32), native.constant(nom_limbs.len() as u32)];
        let modulus_limbs = self.n_const.limbs.clone();
        hint_inputs.extend(modulus_limbs);
        hint_inputs.extend(nom_limbs);
        hint_inputs.extend(denom_limbs);
        native.new_hint("myhint.divhint", &hint_inputs, T::nb_limbs() as usize)
    }
    pub fn mul_const<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, a: &Element<T>, c: BigInt) -> Element<T> {
        if c.is_negative() {
            let neg_a = self.neg(native, a);
            return self.mul_const(native, &neg_a, -c);
        } else if c.is_zero() {
            return self.zero_const.clone();
        }
        let cbl = c.bits();
        if cbl > self.max_overflow() {
            panic!("constant bit length {} exceeds max {}", cbl, self.max_overflow());
        }
        let next_overflow = a.overflow + cbl as u32;
        let mut new_a = a.clone();
        if next_overflow > self.max_of {
            new_a = self.reduce(native, a, false);
        }
        let mut limbs = vec![native.constant(0); new_a.limbs.len()];
        for i in 0..new_a.limbs.len() {
            limbs[i] = native.mul(new_a.limbs[i], c.to_u64().unwrap() as u32);
        }
        let ret = new_internal_element::<T>(limbs, new_a.overflow + cbl as u32);
        return ret;
    }
    pub fn check_mul<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B) {
        let commitment = native.get_random_value();
        // let commitment = native.constant(1); //TBD
        let mut coefs_len = T::nb_limbs() as usize;
        for i in 0..self.mul_checks.len() {
            coefs_len = std::cmp::max(coefs_len, self.mul_checks[i].a.limbs.len());
            coefs_len = std::cmp::max(coefs_len, self.mul_checks[i].b.limbs.len());
            coefs_len = std::cmp::max(coefs_len, self.mul_checks[i].c.limbs.len());
            coefs_len = std::cmp::max(coefs_len, self.mul_checks[i].k.limbs.len());
        }
        let mut at = vec![commitment; coefs_len];
        for i in 1..at.len() {
            at[i] = native.mul(at[i-1], commitment);
        }
        for i in 0..self.mul_checks.len() {
            self.mul_checks[i].eval_round1(native, at.clone());
        }
        for i in 0..self.mul_checks.len() {
            self.mul_checks[i].eval_round2(native, at.clone());
        }
        let pval = eval_with_challenge(native, self.n_const.clone(), at.clone());
        // println!("pval:{:?}", native.value_of(pval.evaluation));
        let coef = BigInt::from(1) << T::bits_per_limb();
        let ccoef = native.sub(coef.to_u64().unwrap() as u32, commitment);
        for i in 0..self.mul_checks.len() {
            // println!("mul_check {}", i);
            self.mul_checks[i].check(native, pval.evaluation, ccoef);
        }
        for i in 0..self.mul_checks.len() {
            self.mul_checks[i].clean_evaluations();
        }
    }
}
pub fn eval_with_challenge<'a, C: Config, B: RootAPI<C>, T: FieldParams>(native: &'a mut B, a: Element<T>, at: Vec<Variable>) -> Element<T> {
    if a.is_evaluated {
        return a;
    }
    if (at.len() as i64) < (a.limbs.len() as i64) - 1 {
        panic!("evaluation powers less than limbs");
    }
    let mut sum = native.constant(0);
    if a.limbs.len() > 0 {
        sum = native.mul(a.limbs[0], 1);
    }
    for i in 1..a.limbs.len() {
        let tmp = native.mul(a.limbs[i], at[i-1]);
        sum = native.add(sum, tmp);
    }
    let mut ret = a.clone();
    ret.is_evaluated = true;
    ret.evaluation = sum;
    ret
}
// pub fn normalize(limbs: Vec<Variable>) -> Vec<Variable> {
//     if limbs.len() < nb_limbs {
//         let mut tail = vec![native.constant(0); nb_limbs - limbs.len()];
//         for i in 0..tail.len() {
//             tail[i] = native.constant(0);
//         }
//         return limbs.iter().chain(tail.iter()).cloned().collect();
//     }
//     limbs
// };