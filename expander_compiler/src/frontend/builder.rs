use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use ethnum::U256;

use crate::{
    circuit::{
        config::Config,
        ir::{
            expr::{LinComb, LinCombTerm},
            source::{self, Constraint as SourceConstraint, Instruction as SourceInstruction},
        },
        layered::Coef,
    },
    field::{Field, FieldArith},
    utils::function_id::get_function_id,
};

use super::api::BasicAPI;

pub struct Builder<C: Config> {
    instructions: Vec<SourceInstruction<C>>,
    constraints: Vec<SourceConstraint>,
    var_max: usize,
    num_inputs: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Variable {
    id: usize,
}

pub enum VariableOrValue<F: Field> {
    Variable(Variable),
    Value(F),
}

pub trait ToVariableOrValue<F: Field> {
    fn convert_to_variable_or_value(self) -> VariableOrValue<F>;
}

trait NotVariable {}
impl NotVariable for u32 {}
impl NotVariable for U256 {}

impl<F: Field, T: Into<F> + NotVariable> ToVariableOrValue<F> for T {
    fn convert_to_variable_or_value(self) -> VariableOrValue<F> {
        VariableOrValue::Value(self.into())
    }
}

impl<F: Field> ToVariableOrValue<F> for Variable {
    fn convert_to_variable_or_value(self) -> VariableOrValue<F> {
        VariableOrValue::Variable(self)
    }
}

impl<F: Field> ToVariableOrValue<F> for &Variable {
    fn convert_to_variable_or_value(self) -> VariableOrValue<F> {
        VariableOrValue::Variable(*self)
    }
}

impl<C: Config> Builder<C> {
    pub fn new(num_inputs: usize) -> (Self, Vec<Variable>) {
        (
            Builder {
                instructions: Vec::new(),
                constraints: Vec::new(),
                var_max: num_inputs,
                num_inputs,
            },
            (1..=num_inputs).map(|id| Variable { id }).collect(),
        )
    }

    pub fn build(self, outputs: &[Variable]) -> source::Circuit<C> {
        source::Circuit {
            instructions: self.instructions,
            constraints: self.constraints,
            num_inputs: self.num_inputs,
            num_hint_inputs: 0,
            outputs: outputs.iter().map(|v| v.id).collect(),
        }
    }

    fn convert_to_variable<T: ToVariableOrValue<C::CircuitField>>(&mut self, value: T) -> Variable {
        match value.convert_to_variable_or_value() {
            VariableOrValue::Variable(v) => v,
            VariableOrValue::Value(v) => {
                self.instructions
                    .push(SourceInstruction::ConstantLike(Coef::Constant(v)));
                self.var_max += 1;
                Variable { id: self.var_max }
            }
        }
    }

    fn new_var(&mut self) -> Variable {
        self.var_max += 1;
        Variable { id: self.var_max }
    }
}

impl<C: Config> BasicAPI<C> for Builder<C> {
    fn add(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) -> Variable {
        let x = self.convert_to_variable(x);
        let y = self.convert_to_variable(y);
        self.instructions.push(SourceInstruction::LinComb(LinComb {
            terms: vec![
                LinCombTerm {
                    var: x.id,
                    coef: C::CircuitField::one(),
                },
                LinCombTerm {
                    var: y.id,
                    coef: C::CircuitField::one(),
                },
            ],
            constant: C::CircuitField::zero(),
        }));
        self.new_var()
    }

    fn sub(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) -> Variable {
        let x = self.convert_to_variable(x);
        let y = self.convert_to_variable(y);
        self.instructions.push(SourceInstruction::LinComb(LinComb {
            terms: vec![
                LinCombTerm {
                    var: x.id,
                    coef: C::CircuitField::one(),
                },
                LinCombTerm {
                    var: y.id,
                    coef: -C::CircuitField::one(),
                },
            ],
            constant: C::CircuitField::zero(),
        }));
        self.new_var()
    }

    fn neg(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable {
        let x = self.convert_to_variable(x);
        self.instructions.push(SourceInstruction::LinComb(LinComb {
            terms: vec![LinCombTerm {
                var: x.id,
                coef: -C::CircuitField::one(),
            }],
            constant: C::CircuitField::zero(),
        }));
        self.new_var()
    }

    fn mul(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) -> Variable {
        let x = self.convert_to_variable(x);
        let y = self.convert_to_variable(y);
        self.instructions
            .push(SourceInstruction::Mul(vec![x.id, y.id]));
        self.new_var()
    }

    fn div(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
        checked: bool,
    ) -> Variable {
        let x = self.convert_to_variable(x);
        let y = self.convert_to_variable(y);
        self.instructions.push(SourceInstruction::Div {
            x: x.id,
            y: y.id,
            checked,
        });
        self.new_var()
    }

    fn inverse(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable {
        self.div(1, x, true)
    }

    fn xor(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) -> Variable {
        let x = self.convert_to_variable(x);
        let y = self.convert_to_variable(y);
        self.instructions.push(SourceInstruction::BoolBinOp {
            x: x.id,
            y: y.id,
            op: source::BoolBinOpType::Xor,
        });
        self.new_var()
    }

    fn or(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) -> Variable {
        let x = self.convert_to_variable(x);
        let y = self.convert_to_variable(y);
        self.instructions.push(SourceInstruction::BoolBinOp {
            x: x.id,
            y: y.id,
            op: source::BoolBinOpType::Or,
        });
        self.new_var()
    }

    fn and(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) -> Variable {
        let x = self.convert_to_variable(x);
        let y = self.convert_to_variable(y);
        self.instructions.push(SourceInstruction::BoolBinOp {
            x: x.id,
            y: y.id,
            op: source::BoolBinOpType::And,
        });
        self.new_var()
    }

    fn is_zero(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable {
        let x = self.convert_to_variable(x);
        self.instructions.push(SourceInstruction::IsZero(x.id));
        self.new_var()
    }

    fn assert_is_zero(&mut self, x: impl ToVariableOrValue<C::CircuitField>) {
        let x = self.convert_to_variable(x);
        self.constraints.push(SourceConstraint {
            typ: source::ConstraintType::Zero,
            var: x.id,
        });
    }

    fn assert_is_non_zero(&mut self, x: impl ToVariableOrValue<C::CircuitField>) {
        let x = self.convert_to_variable(x);
        self.constraints.push(SourceConstraint {
            typ: source::ConstraintType::NonZero,
            var: x.id,
        });
    }

    fn assert_is_bool(&mut self, x: impl ToVariableOrValue<C::CircuitField>) {
        let x = self.convert_to_variable(x);
        self.constraints.push(SourceConstraint {
            typ: source::ConstraintType::Bool,
            var: x.id,
        });
    }

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
}

pub struct RootBuilder<C: Config> {
    num_public_inputs: usize,
    current_builders: Vec<(usize, Builder<C>)>,
    sub_circuits: HashMap<usize, source::Circuit<C>>,
}

impl<C: Config> BasicAPI<C> for RootBuilder<C> {
    fn add(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) -> Variable {
        self.last_builder().add(x, y)
    }

    fn sub(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) -> Variable {
        self.last_builder().sub(x, y)
    }

    fn neg(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable {
        self.last_builder().neg(x)
    }

    fn mul(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) -> Variable {
        self.last_builder().mul(x, y)
    }

    fn div(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
        checked: bool,
    ) -> Variable {
        self.last_builder().div(x, y, checked)
    }

    fn inverse(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable {
        self.last_builder().inverse(x)
    }

    fn xor(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) -> Variable {
        self.last_builder().xor(x, y)
    }

    fn or(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) -> Variable {
        self.last_builder().or(x, y)
    }

    fn and(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) -> Variable {
        self.last_builder().and(x, y)
    }

    fn is_zero(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable {
        self.last_builder().is_zero(x)
    }

    fn assert_is_zero(&mut self, x: impl ToVariableOrValue<C::CircuitField>) {
        self.last_builder().assert_is_zero(x)
    }

    fn assert_is_non_zero(&mut self, x: impl ToVariableOrValue<C::CircuitField>) {
        self.last_builder().assert_is_non_zero(x)
    }

    fn assert_is_bool(&mut self, x: impl ToVariableOrValue<C::CircuitField>) {
        self.last_builder().assert_is_bool(x)
    }

    fn assert_is_equal(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) {
        self.last_builder().assert_is_equal(x, y)
    }

    fn assert_is_different(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) {
        self.last_builder().assert_is_different(x, y)
    }
}

impl<C: Config> RootBuilder<C> {
    pub fn new(
        num_inputs: usize,
        num_public_inputs: usize,
    ) -> (Self, Vec<Variable>, Vec<Variable>) {
        let (mut builder0, inputs) = Builder::new(num_inputs);
        let public_inputs = (0..num_public_inputs).map(|_| builder0.new_var()).collect();
        for i in 0..num_public_inputs {
            builder0
                .instructions
                .push(SourceInstruction::ConstantLike(Coef::PublicInput(i)));
        }
        (
            RootBuilder {
                num_public_inputs,
                current_builders: vec![(0, builder0)],
                sub_circuits: HashMap::new(),
            },
            inputs,
            public_inputs,
        )
    }

    pub fn build(self) -> source::RootCircuit<C> {
        let mut circuits = self.sub_circuits;
        assert_eq!(self.current_builders.len(), 1);
        for (circuit_id, builder) in self.current_builders {
            circuits.insert(circuit_id, builder.build(&[]));
        }
        source::RootCircuit {
            circuits,
            num_public_inputs: self.num_public_inputs,
            expected_num_output_zeroes: 0,
        }
    }

    fn last_builder(&mut self) -> &mut Builder<C> {
        &mut self.current_builders.last_mut().unwrap().1
    }

    fn actually_call_sub_circuit<F: Fn(&mut Self, &Vec<Variable>) -> Vec<Variable>>(
        &mut self,
        circuit_id: usize,
        n: usize,
        f: F,
    ) {
        let (sub_builder, sub_inputs) = Builder::new(n);
        self.current_builders.push((circuit_id, sub_builder));
        let sub_outputs = f(self, &sub_inputs);
        let (_, sub_builder) = self.current_builders.pop().unwrap();
        let sub = sub_builder.build(&sub_outputs);
        self.sub_circuits.insert(circuit_id, sub);
    }

    fn call_sub_circuit<F: Fn(&mut Self, &Vec<Variable>) -> Vec<Variable>>(
        &mut self,
        circuit_id: usize,
        inputs: &[Variable],
        f: F,
    ) -> Vec<Variable> {
        if !self.sub_circuits.contains_key(&circuit_id) {
            self.actually_call_sub_circuit(circuit_id, inputs.len(), f);
        }
        let sub = self.sub_circuits.get(&circuit_id).unwrap();
        let outputs: Vec<Variable> = (0..sub.outputs.len())
            .map(|_| self.last_builder().new_var())
            .collect();
        self.last_builder()
            .instructions
            .push(SourceInstruction::SubCircuitCall {
                sub_circuit_id: circuit_id,
                inputs: inputs.iter().map(|v| v.id).collect(),
                num_outputs: outputs.len(),
            });
        outputs
    }

    pub fn memorized_simple_call<F: Fn(&mut Self, &Vec<Variable>) -> Vec<Variable> + 'static>(
        &mut self,
        f: F,
        inputs: &[Variable],
    ) -> Vec<Variable> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "simple".hash(&mut hasher);
        inputs.len().hash(&mut hasher);
        get_function_id::<F>().hash(&mut hasher);
        let circuit_id = hasher.finish() as usize;
        self.call_sub_circuit(circuit_id, inputs, f)
    }

    pub fn constant<T: ToVariableOrValue<C::CircuitField>>(&mut self, value: T) -> Variable {
        self.last_builder().convert_to_variable(value)
    }
}
