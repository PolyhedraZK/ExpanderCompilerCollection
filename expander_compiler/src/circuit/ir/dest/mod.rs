use std::collections::{HashMap, HashSet};

use crate::circuit::{config::Config, layered::Coef};
use crate::field::Field;

use super::common::EvalResult;
use super::expr::{Term, VarSpec};
use super::{
    common::{self, Instruction as _, IrConfig, RawConstraint},
    expr::Expression,
};

#[cfg(test)]
pub mod tests;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Instruction<C: Config> {
    InternalVariable {
        expr: Expression<C>,
    },
    SubCircuitCall {
        sub_circuit_id: usize,
        inputs: Vec<usize>,
        num_outputs: usize,
    },
    ConstantOrRandom {
        value: Coef<C>,
    },
}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Irc<C: Config> {
    _a: C,
}
impl<C: Config> IrConfig for Irc<C> {
    type Instruction = Instruction<C>;
    type Constraint = RawConstraint;
    type Config = C;
    const ALLOW_DUPLICATE_SUB_CIRCUIT_INPUTS: bool = false;
    const ALLOW_DUPLICATE_CONSTRAINTS: bool = true;
    const ALLOW_DUPLICATE_OUTPUTS: bool = false;
    const HAS_HINT_INPUT: bool = true;
}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct IrcRelaxed<C: Config> {
    _a: C,
}
impl<C: Config> IrConfig for IrcRelaxed<C> {
    type Instruction = Instruction<C>;
    type Constraint = RawConstraint;
    type Config = C;
    const ALLOW_DUPLICATE_SUB_CIRCUIT_INPUTS: bool = true;
    const ALLOW_DUPLICATE_CONSTRAINTS: bool = true;
    const ALLOW_DUPLICATE_OUTPUTS: bool = true;
    const HAS_HINT_INPUT: bool = true;
}

impl<C: Config> common::Instruction<C> for Instruction<C> {
    fn inputs(&self) -> Vec<usize> {
        match self {
            Instruction::InternalVariable { expr } => expr.get_vars(),
            Instruction::SubCircuitCall { inputs, .. } => inputs.clone(),
            Instruction::ConstantOrRandom { .. } => Vec::new(),
        }
    }
    fn num_outputs(&self) -> usize {
        match self {
            Instruction::InternalVariable { .. } => 1,
            Instruction::SubCircuitCall { num_outputs, .. } => *num_outputs,
            Instruction::ConstantOrRandom { .. } => 1,
        }
    }
    fn as_sub_circuit_call(&self) -> Option<(usize, &Vec<usize>, usize)> {
        match self {
            Instruction::SubCircuitCall {
                sub_circuit_id,
                inputs,
                num_outputs,
            } => Some((*sub_circuit_id, &inputs, *num_outputs)),
            _ => None,
        }
    }
    fn sub_circuit_call(sub_circuit_id: usize, inputs: Vec<usize>, num_outputs: usize) -> Self {
        Instruction::SubCircuitCall {
            sub_circuit_id,
            inputs,
            num_outputs,
        }
    }
    fn replace_vars<F: Fn(usize) -> usize>(&self, f: F) -> Self {
        match self {
            Instruction::InternalVariable { expr } => Instruction::InternalVariable {
                expr: expr.replace_vars(f),
            },
            Instruction::SubCircuitCall {
                sub_circuit_id,
                inputs,
                num_outputs,
            } => Instruction::SubCircuitCall {
                sub_circuit_id: *sub_circuit_id,
                inputs: inputs.iter().map(|&i| f(i)).collect(),
                num_outputs: *num_outputs,
            },
            Instruction::ConstantOrRandom { value } => Instruction::ConstantOrRandom {
                value: value.clone(),
            },
        }
    }
    fn from_kx_plus_b(x: usize, k: C::CircuitField, b: C::CircuitField) -> Self {
        Instruction::InternalVariable {
            expr: Expression::from_terms(vec![Term::new_linear(k, x), Term::new_const(b)]),
        }
    }
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
    fn eval_unsafe(&self, values: &[C::CircuitField]) -> EvalResult<C> {
        match self {
            Instruction::InternalVariable { expr } => {
                let mut sum = C::CircuitField::zero();
                for term in expr.iter() {
                    match &term.vars {
                        VarSpec::Const => {
                            sum += term.coef;
                        }
                        VarSpec::Linear(i) => {
                            sum += values[*i] * term.coef;
                        }
                        VarSpec::Quad(i, j) => {
                            sum += values[*i] * values[*j] * term.coef;
                        }
                    }
                }
                EvalResult::Value(sum)
            }
            Instruction::SubCircuitCall {
                sub_circuit_id,
                inputs,
                ..
            } => EvalResult::SubCircuitCall(*sub_circuit_id, inputs),
            Instruction::ConstantOrRandom { value } => EvalResult::Value(value.get_value_unsafe()),
        }
    }
}

pub type Circuit<C> = common::Circuit<Irc<C>>;
pub type RootCircuit<C> = common::RootCircuit<Irc<C>>;
pub type CircuitRelaxed<C> = common::Circuit<IrcRelaxed<C>>;
pub type RootCircuitRelaxed<C> = common::RootCircuit<IrcRelaxed<C>>;

impl<C: Config> CircuitRelaxed<C> {
    fn adjust_for_layering(&self) -> Circuit<C> {
        let mut new_id = vec![0];
        let mut new_instructions: Vec<Instruction<C>> = Vec::new();
        let mut insn_of_var = vec![None];
        let mut new_var_max = self.get_num_inputs_all();
        for i in 1..=self.get_num_inputs_all() {
            new_id.push(i);
            insn_of_var.push(None);
        }
        for insn in self.instructions.iter() {
            let new_insn = match insn.replace_vars(|x| new_id[x]) {
                Instruction::InternalVariable { expr } => {
                    insn_of_var.push(Some(new_instructions.len()));
                    Instruction::InternalVariable { expr: expr }
                }
                Instruction::ConstantOrRandom { value } => {
                    insn_of_var.push(Some(new_instructions.len()));
                    Instruction::ConstantOrRandom { value: value }
                }
                Instruction::SubCircuitCall {
                    sub_circuit_id,
                    inputs,
                    num_outputs,
                } => {
                    let mut occured = HashSet::new();
                    let mut new_inputs = Vec::new();
                    for &i in inputs.iter() {
                        if !occured.contains(&i) {
                            occured.insert(i);
                            new_inputs.push(i);
                        } else {
                            let copy_insn = match &insn_of_var[i] {
                                Some(insn_id) => new_instructions[*insn_id].clone(),
                                None => Instruction::InternalVariable {
                                    expr: Expression::new_linear(C::CircuitField::one(), i),
                                },
                            };
                            new_var_max += 1;
                            new_instructions.push(copy_insn);
                            new_inputs.push(new_var_max);
                            insn_of_var.push(None);
                        }
                    }
                    for _ in 0..num_outputs {
                        insn_of_var.push(None);
                    }
                    Instruction::SubCircuitCall {
                        sub_circuit_id: sub_circuit_id,
                        inputs: new_inputs,
                        num_outputs: num_outputs,
                    }
                }
            };
            for _ in 0..new_insn.num_outputs() {
                new_var_max += 1;
                new_id.push(new_var_max);
            }
            new_instructions.push(new_insn);
        }
        let outputs_set: HashSet<usize> = self.outputs.iter().cloned().collect();
        if outputs_set.len() != self.outputs.len() {
            panic!("unexpected situation: duplicate outputs should be removed in previous optimization pass");
        }
        let new_outputs: Vec<usize> = self.outputs.iter().map(|&o| new_id[o]).collect();
        let constraints_set: HashSet<usize> = self.constraints.iter().map(|&c| new_id[c]).collect();
        let new_constraints: Vec<usize> = constraints_set.iter().cloned().collect();
        Circuit {
            instructions: new_instructions,
            outputs: new_outputs,
            constraints: new_constraints,
            num_inputs: self.num_inputs,
            num_hint_inputs: self.num_hint_inputs,
        }
    }
}

impl<C: Config> RootCircuitRelaxed<C> {
    pub fn adjust_for_layering(&self) -> RootCircuit<C> {
        let mut new_circuits = HashMap::new();
        for (id, circuit) in self.circuits.iter() {
            new_circuits.insert(*id, circuit.adjust_for_layering());
        }
        RootCircuit {
            circuits: new_circuits,
        }
    }
}
