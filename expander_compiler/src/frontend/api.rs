use crate::circuit::config::Config;

use super::builder::{ToVariableOrValue, Variable};

macro_rules! binary_op {
    ($name:ident) => {
        fn $name(
            &mut self,
            x: impl ToVariableOrValue<C::CircuitField>,
            y: impl ToVariableOrValue<C::CircuitField>,
        ) -> Variable;
    };
}

pub trait BasicAPI<C: Config> {
    binary_op!(add);
    binary_op!(sub);
    binary_op!(mul);
    binary_op!(xor);
    binary_op!(or);
    binary_op!(and);

    fn display(&self, _label: &str, _x: impl ToVariableOrValue<C::CircuitField>) {}
    fn div(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
        checked: bool,
    ) -> Variable;
    fn neg(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn inverse(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable {
        self.div(1, x, true)
    }
    fn is_zero(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn assert_is_zero(&mut self, x: impl ToVariableOrValue<C::CircuitField>);
    fn assert_is_non_zero(&mut self, x: impl ToVariableOrValue<C::CircuitField>);
    fn assert_is_bool(&mut self, x: impl ToVariableOrValue<C::CircuitField>);
    fn assert_is_equal(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) {
        let diff = self.sub(x, y);
        self.assert_is_zero(diff);
    }
    fn assert_is_different(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) {
        let diff = self.sub(x, y);
        self.assert_is_non_zero(diff);
    }
    fn get_random_value(&mut self) -> Variable;
    fn new_hint(
        &mut self,
        hint_key: &str,
        inputs: &[Variable],
        num_outputs: usize,
    ) -> Vec<Variable>;
    fn constant(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    // try to get the value of a compile-time constant variable
    // this function has different behavior in normal and debug mode, in debug mode it always returns Some(value)
    fn constant_value(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
    ) -> Option<C::CircuitField>;
}

pub trait UnconstrainedAPI<C: Config> {
    fn unconstrained_identity(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    binary_op!(unconstrained_add);
    binary_op!(unconstrained_mul);
    binary_op!(unconstrained_div);
    binary_op!(unconstrained_pow);
    binary_op!(unconstrained_int_div);
    binary_op!(unconstrained_mod);
    binary_op!(unconstrained_shift_l);
    binary_op!(unconstrained_shift_r);
    binary_op!(unconstrained_lesser_eq);
    binary_op!(unconstrained_greater_eq);
    binary_op!(unconstrained_lesser);
    binary_op!(unconstrained_greater);
    binary_op!(unconstrained_eq);
    binary_op!(unconstrained_not_eq);
    binary_op!(unconstrained_bool_or);
    binary_op!(unconstrained_bool_and);
    binary_op!(unconstrained_bit_or);
    binary_op!(unconstrained_bit_and);
    binary_op!(unconstrained_bit_xor);
}

pub trait RootAPI<C: Config>: Sized + BasicAPI<C> + UnconstrainedAPI<C> + 'static {
    fn memorized_simple_call<F: Fn(&mut Self, &Vec<Variable>) -> Vec<Variable> + 'static>(
        &mut self,
        f: F,
        inputs: &[Variable],
    ) -> Vec<Variable>;
    fn hash_to_sub_circuit_id(&mut self, hash: &[u8; 32]) -> usize;
    // This function should only be called in proc macro generated code
    fn call_sub_circuit<F: FnOnce(&mut Self, &Vec<Variable>) -> Vec<Variable>>(
        &mut self,
        circuit_id: usize,
        inputs: &[Variable],
        f: F,
    ) -> Vec<Variable>;
    // This function should only be called in proc macro generated code
    fn register_sub_circuit_output_structure(&mut self, circuit_id: usize, structure: Vec<usize>);
    // This function should only be called in proc macro generated code
    fn get_sub_circuit_output_structure(&self, circuit_id: usize) -> Vec<usize>;
    fn set_outputs(&mut self, outputs: Vec<Variable>);
}
