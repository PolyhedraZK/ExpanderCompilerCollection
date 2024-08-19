use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

use crate::{
    circuit::config::Config,
    field::Field,
    utils::{
        error::Error,
        misc::{topo_order, topo_order_and_is_dag},
    },
};

pub mod display;
pub mod opt;
pub mod serde;
pub mod stats;

#[cfg(test)]
pub mod rand_gen;

pub trait IrConfig: Debug + Clone + Default + Hash + PartialEq + Eq {
    type Config: Config;
    type Instruction: Instruction<Self::Config>;
    type Constraint: Constraint<Self::Config>;
    const ALLOW_DUPLICATE_SUB_CIRCUIT_INPUTS: bool;
    const ALLOW_DUPLICATE_CONSTRAINTS: bool;
    const ALLOW_DUPLICATE_OUTPUTS: bool;
    const HAS_HINT_INPUT: bool;
}

pub trait Instruction<C: Config>: Debug + Clone + Hash + PartialEq + Eq {
    fn inputs(&self) -> Vec<usize>;
    fn num_outputs(&self) -> usize;
    fn as_sub_circuit_call(&self) -> Option<(usize, &Vec<usize>, usize)>;
    fn sub_circuit_call(sub_circuit_id: usize, inputs: Vec<usize>, num_outputs: usize) -> Self;
    fn replace_vars<F: Fn(usize) -> usize>(&self, f: F) -> Self;
    fn from_kx_plus_b(x: usize, k: C::CircuitField, b: C::CircuitField) -> Self;
    fn validate(&self) -> Result<(), Error>;
    fn eval_unsafe(&self, values: &[C::CircuitField]) -> EvalResult<'_, C>;
}

pub enum EvalResult<'a, C: Config> {
    Value(C::CircuitField),
    Values(Vec<C::CircuitField>),
    SubCircuitCall(usize, &'a Vec<usize>),
    Error(Error),
}

pub trait Constraint<C: Config>: Debug + Clone + Hash + PartialEq + Eq {
    type Type: ConstraintType<C>;
    fn var(&self) -> usize;
    fn typ(&self) -> Self::Type;
    fn replace_var<F: Fn(usize) -> usize>(&self, f: F) -> Self;
    fn new(var: usize, typ: Self::Type) -> Self;
}

pub trait ConstraintType<C: Config>: Debug + Copy + Clone + Hash + PartialEq + Eq {
    fn verify(&self, value: &C::CircuitField) -> bool;
}

pub type RawConstraint = usize;
pub type RawConstraintType = ();

impl<C: Config> Constraint<C> for RawConstraint {
    type Type = RawConstraintType;
    fn var(&self) -> usize {
        *self
    }
    fn typ(&self) -> Self::Type {
        ()
    }
    fn replace_var<F: Fn(usize) -> usize>(&self, f: F) -> Self {
        f(*self)
    }
    fn new(var: usize, _: Self::Type) -> Self {
        var
    }
}

impl<C: Config> ConstraintType<C> for RawConstraintType {
    fn verify(&self, x: &C::CircuitField) -> bool {
        x.is_zero()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Circuit<Irc: IrConfig> {
    pub instructions: Vec<Irc::Instruction>,
    pub constraints: Vec<Irc::Constraint>,
    pub outputs: Vec<usize>,
    pub num_inputs: usize,
    pub num_hint_inputs: usize,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct RootCircuit<Irc: IrConfig> {
    pub circuits: HashMap<usize, Circuit<Irc>>,
}

impl<Irc: IrConfig> Circuit<Irc> {
    pub fn get_num_inputs_all(&self) -> usize {
        if Irc::HAS_HINT_INPUT {
            self.num_inputs + self.num_hint_inputs
        } else {
            self.num_inputs
        }
    }

    fn validate_variable_references(&self) -> Result<(), Error> {
        if !Irc::HAS_HINT_INPUT && self.num_hint_inputs != 0 {
            return Err(Error::InternalError(
                "hint input is not allowed".to_string(),
            ));
        }
        let mut cur_var_max = self.get_num_inputs_all();
        for insn in self.instructions.iter() {
            for term in insn.inputs() {
                if term > cur_var_max || term == 0 {
                    return Err(Error::InternalError(format!(
                        "invalid variable reference: {}",
                        term
                    )));
                }
            }
            insn.validate()?;
            cur_var_max += insn.num_outputs();
            if !Irc::ALLOW_DUPLICATE_SUB_CIRCUIT_INPUTS {
                if let Some((_, inputs, _)) = insn.as_sub_circuit_call() {
                    let mut set = HashSet::new();
                    for &input in inputs.iter() {
                        if !set.insert(input) {
                            return Err(Error::InternalError(format!(
                                "duplicate sub circuit input: {}",
                                input
                            )));
                        }
                    }
                }
            }
        }
        for c in self.constraints.iter() {
            if c.var() > cur_var_max || c.var() == 0 {
                return Err(Error::InternalError(format!(
                    "invalid constraint variable reference: {}",
                    c.var()
                )));
            }
        }
        if !Irc::ALLOW_DUPLICATE_CONSTRAINTS {
            let mut set = HashSet::new();
            for c in self.constraints.iter() {
                if !set.insert(c.var()) {
                    return Err(Error::InternalError(format!(
                        "duplicate constraint: {}",
                        c.var()
                    )));
                }
            }
        }
        for &output in self.outputs.iter() {
            if output > cur_var_max || output == 0 {
                return Err(Error::InternalError(format!(
                    "invalid output variable reference: {}",
                    output
                )));
            }
        }
        if !Irc::ALLOW_DUPLICATE_OUTPUTS {
            let mut set = HashSet::new();
            for &output in self.outputs.iter() {
                if !set.insert(output) {
                    return Err(Error::InternalError(format!(
                        "duplicate output: {}",
                        output
                    )));
                }
            }
        }
        Ok(())
    }

    pub fn get_num_variables(&self) -> usize {
        let mut cur_var_max = self.get_num_inputs_all();
        for insn in self.instructions.iter() {
            cur_var_max += insn.num_outputs();
        }
        cur_var_max
    }
}

impl<Irc: IrConfig> RootCircuit<Irc> {
    pub fn sub_circuit_graph_vertices(&self) -> HashSet<usize> {
        self.circuits.keys().cloned().collect()
    }

    pub fn sub_circuit_graph_edges(&self) -> HashMap<usize, HashSet<usize>> {
        let mut edges: HashMap<usize, HashSet<usize>> = HashMap::new();
        for (circuit_id, circuit) in self.circuits.iter() {
            for insn in circuit.instructions.iter() {
                if let Some((sub_circuit_id, _, _)) = insn.as_sub_circuit_call() {
                    edges
                        .entry(*circuit_id)
                        .or_insert(HashSet::new())
                        .insert(sub_circuit_id);
                }
            }
        }
        edges
    }

    pub fn validate(&self) -> Result<(), Error> {
        // tests of this function are in for_layering
        // check if 0 circuit exists
        if !self.circuits.contains_key(&0) {
            return Err(Error::InternalError("root circuit not found".to_string()));
        }
        // check if all variable references are valid
        for circuit in self.circuits.values() {
            circuit.validate_variable_references()?;
        }
        // check if all sub circuit calls are valid and it's a DAG
        for circuit in self.circuits.values() {
            for insn in circuit.instructions.iter() {
                if let Some((sub_circuit_id, inputs, num_outputs)) = insn.as_sub_circuit_call() {
                    if !self.circuits.contains_key(&sub_circuit_id) {
                        return Err(Error::InternalError(format!(
                            "sub circuit {} not found",
                            sub_circuit_id
                        )));
                    }
                    if inputs.len() != self.circuits[&sub_circuit_id].num_inputs {
                        return Err(Error::InternalError(format!(
                            "sub circuit {} expects {} inputs, got {}",
                            sub_circuit_id,
                            self.circuits[&sub_circuit_id].num_inputs,
                            inputs.len()
                        )));
                    }
                    if num_outputs != self.circuits[&sub_circuit_id].outputs.len() {
                        return Err(Error::InternalError(format!(
                            "sub circuit {} expects {} outputs, got {}",
                            sub_circuit_id,
                            self.circuits[&sub_circuit_id].outputs.len(),
                            num_outputs
                        )));
                    }
                }
            }
        }
        let s_edges = self.sub_circuit_graph_edges();
        let (order, is_dag) = topo_order_and_is_dag(&self.sub_circuit_graph_vertices(), &s_edges);
        if !is_dag {
            return Err(Error::InternalError("circuit is not a DAG".to_string()));
        }
        // check if root circuit has constraint
        let mut has_constraint: HashSet<usize> = HashSet::new();
        for circuit_id in order.iter().rev() {
            let circuit = &self.circuits[circuit_id];
            if !circuit.constraints.is_empty() {
                has_constraint.insert(*circuit_id);
                continue;
            }
            if let Some(e) = s_edges.get(circuit_id) {
                for o in e {
                    if has_constraint.contains(o) {
                        has_constraint.insert(*circuit_id);
                        break;
                    }
                }
            }
        }
        if !has_constraint.contains(&0) {
            return Err(Error::UserError(
                "root circuit should have constraints".to_string(),
            ));
        }
        Ok(())
    }

    pub fn input_size(&self) -> usize {
        // tests of this function are in for_layering
        if !Irc::HAS_HINT_INPUT {
            return self.circuits[&0].num_inputs;
        }
        let order = self.topo_order();
        let mut sub_hint_size: HashMap<usize, usize> = HashMap::new();
        for i in order.iter().rev() {
            let circuit = &self.circuits[i];
            let mut hint_size = circuit.num_hint_inputs;
            for insn in circuit.instructions.iter() {
                if let Some((sub_circuit_id, _, _)) = insn.as_sub_circuit_call() {
                    hint_size += sub_hint_size[&sub_circuit_id];
                }
            }
            sub_hint_size.insert(*i, hint_size);
        }
        return self.circuits[&0].num_inputs + sub_hint_size[&0];
    }

    pub fn topo_order(&self) -> Vec<usize> {
        topo_order(
            &self.sub_circuit_graph_vertices(),
            &self.sub_circuit_graph_edges(),
        )
    }

    // eval the circuit. This function should be used for testing only
    pub fn eval_unsafe_with_errors(
        &self,
        inputs: Vec<<Irc::Config as Config>::CircuitField>,
    ) -> Result<(Vec<<Irc::Config as Config>::CircuitField>, bool), Error> {
        assert_eq!(inputs.len(), self.input_size());
        let (root_input, hint_input) = inputs.split_at(self.circuits[&0].num_inputs);
        let (res, rem, cond) =
            self.eval_unsafe_sub(&self.circuits[&0], root_input.to_vec(), hint_input)?;
        assert_eq!(rem.len(), 0);
        Ok((res, cond))
    }

    pub fn eval_unsafe(
        &self,
        inputs: Vec<<Irc::Config as Config>::CircuitField>,
    ) -> (Vec<<Irc::Config as Config>::CircuitField>, bool) {
        self.eval_unsafe_with_errors(inputs).unwrap()
    }

    fn eval_unsafe_sub<'a>(
        &self,
        circuit: &Circuit<Irc>,
        inputs: Vec<<Irc::Config as Config>::CircuitField>,
        hint_inputs: &'a [<Irc::Config as Config>::CircuitField],
    ) -> Result<
        (
            Vec<<Irc::Config as Config>::CircuitField>,
            &'a [<Irc::Config as Config>::CircuitField],
            bool,
        ),
        Error,
    > {
        let mut values = vec![<Irc::Config as Config>::CircuitField::zero(); 1];
        values.extend(inputs);
        let (cur_hint_input, rem_hint_inputs) = hint_inputs.split_at(circuit.num_hint_inputs);
        values.extend(cur_hint_input);
        let mut cond = true;
        let mut hint_inputs = rem_hint_inputs;
        for insn in circuit.instructions.iter() {
            match insn.eval_unsafe(&values) {
                EvalResult::Value(v) => {
                    values.push(v);
                }
                EvalResult::Values(mut vs) => {
                    values.append(&mut vs);
                }
                EvalResult::SubCircuitCall(sub_circuit_id, inputs) => {
                    let (res, rem, sub_cond) = self.eval_unsafe_sub(
                        &self.circuits[&sub_circuit_id],
                        inputs.iter().map(|&i| values[i]).collect(),
                        hint_inputs,
                    )?;
                    hint_inputs = rem;
                    values.extend(res);
                    cond &= sub_cond;
                }
                EvalResult::Error(e) => {
                    return Err(e);
                }
            }
        }
        for c in circuit.constraints.iter() {
            cond &= c.typ().verify(&values[c.var()]);
        }
        let mut res = Vec::new();
        for &o in circuit.outputs.iter() {
            res.push(values[o]);
        }
        Ok((res, hint_inputs, cond))
    }
}