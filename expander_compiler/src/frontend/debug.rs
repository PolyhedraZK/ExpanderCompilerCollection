use gkr_engine::FieldEngine;

use crate::{
    circuit::{
        config::Config,
        ir::{
            common::{EvalResult, Instruction},
            source::{BoolBinOpType, Instruction as IrInstruction, UnconstrainedBinOpType},
        },
    },
    field::FieldArith,
    hints::registry::{hint_key_to_id, HintCaller},
};

use super::{
    api::{BasicAPI, RootAPI, UnconstrainedAPI},
    builder::{
        ensure_variables_valid, get_variable_id, new_variable, ToVariableOrValue, VariableOrValue,
    },
    CircuitField, Field, Variable,
};

pub struct DebugBuilder<C: Config, H: HintCaller<CircuitField<C>>> {
    values: Vec<CircuitField<C>>,
    outputs: Vec<Variable>,
    hint_caller: H,
}

impl<C: Config, H: HintCaller<CircuitField<C>>> BasicAPI<C> for DebugBuilder<C, H> {
    fn display(&self, str: &str, x: impl ToVariableOrValue<CircuitField<C>>) {
        let x = self.convert_to_value(x);
        println!("{}: {:?}", str, x);
    }

    fn add(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_value(x);
        let y = self.convert_to_value(y);
        self.return_as_variable(x + y)
    }
    fn sub(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_value(x);
        let y = self.convert_to_value(y);
        self.return_as_variable(x - y)
    }
    fn mul(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_value(x);
        let y = self.convert_to_value(y);
        self.return_as_variable(x * y)
    }
    fn xor(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::BoolBinOp {
            x,
            y,
            op: BoolBinOpType::Xor,
        })
    }
    fn or(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::BoolBinOp {
            x,
            y,
            op: BoolBinOpType::Or,
        })
    }
    fn and(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::BoolBinOp {
            x,
            y,
            op: BoolBinOpType::And,
        })
    }
    fn div(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
        checked: bool,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::Div { x, y, checked })
    }
    fn neg(&mut self, x: impl ToVariableOrValue<CircuitField<C>>) -> Variable {
        let x = self.convert_to_value(x);
        self.return_as_variable(-x)
    }
    fn is_zero(&mut self, x: impl ToVariableOrValue<CircuitField<C>>) -> Variable {
        let x = self.convert_to_id(x);
        self.eval_ir_insn(IrInstruction::IsZero(x))
    }
    fn assert_is_zero(&mut self, x: impl ToVariableOrValue<CircuitField<C>>) {
        let x = self.convert_to_value(x);
        assert!(x.is_zero());
    }
    fn assert_is_non_zero(&mut self, x: impl ToVariableOrValue<CircuitField<C>>) {
        let x = self.convert_to_value(x);
        assert!(!x.is_zero());
    }
    fn assert_is_bool(&mut self, x: impl ToVariableOrValue<CircuitField<C>>) {
        let x = self.convert_to_value(x);
        assert!(x.is_zero() || x == CircuitField::<C>::one());
    }
    fn get_random_value(&mut self) -> Variable {
        let v = CircuitField::<C>::random_unsafe(&mut rand::thread_rng());
        self.return_as_variable(v)
    }
    fn new_hint(
        &mut self,
        hint_key: &str,
        inputs: &[Variable],
        num_outputs: usize,
    ) -> Vec<Variable> {
        ensure_variables_valid(inputs);
        let inputs: Vec<CircuitField<C>> =
            inputs.iter().map(|v| self.convert_to_value(v)).collect();
        match self
            .hint_caller
            .call(hint_key_to_id(hint_key), &inputs, num_outputs)
        {
            Ok(outputs) => outputs
                .into_iter()
                .map(|v| self.return_as_variable(v))
                .collect(),
            Err(e) => panic!("Hint error: {:?}", e),
        }
    }
    fn constant(&mut self, x: impl ToVariableOrValue<CircuitField<C>>) -> Variable {
        let x = self.convert_to_value(x);
        self.return_as_variable(x)
    }
    fn constant_value(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Option<CircuitField<C>> {
        Some(self.convert_to_value(x))
    }
}

impl<C: Config, H: HintCaller<CircuitField<C>>> UnconstrainedAPI<C> for DebugBuilder<C, H> {
    fn unconstrained_identity(&mut self, x: impl ToVariableOrValue<CircuitField<C>>) -> Variable {
        self.constant(x)
    }
    fn unconstrained_add(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        self.add(x, y)
    }
    fn unconstrained_mul(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        self.mul(x, y)
    }
    fn unconstrained_div(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::UnconstrainedBinOp {
            x,
            y,
            op: UnconstrainedBinOpType::Div,
        })
    }
    fn unconstrained_pow(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::UnconstrainedBinOp {
            x,
            y,
            op: UnconstrainedBinOpType::Pow,
        })
    }
    fn unconstrained_int_div(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::UnconstrainedBinOp {
            x,
            y,
            op: UnconstrainedBinOpType::IntDiv,
        })
    }
    fn unconstrained_mod(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::UnconstrainedBinOp {
            x,
            y,
            op: UnconstrainedBinOpType::Mod,
        })
    }
    fn unconstrained_shift_l(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::UnconstrainedBinOp {
            x,
            y,
            op: UnconstrainedBinOpType::ShiftL,
        })
    }
    fn unconstrained_shift_r(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::UnconstrainedBinOp {
            x,
            y,
            op: UnconstrainedBinOpType::ShiftR,
        })
    }
    fn unconstrained_lesser_eq(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::UnconstrainedBinOp {
            x,
            y,
            op: UnconstrainedBinOpType::LesserEq,
        })
    }
    fn unconstrained_greater_eq(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::UnconstrainedBinOp {
            x,
            y,
            op: UnconstrainedBinOpType::GreaterEq,
        })
    }
    fn unconstrained_lesser(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::UnconstrainedBinOp {
            x,
            y,
            op: UnconstrainedBinOpType::Lesser,
        })
    }
    fn unconstrained_greater(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::UnconstrainedBinOp {
            x,
            y,
            op: UnconstrainedBinOpType::Greater,
        })
    }
    fn unconstrained_eq(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::UnconstrainedBinOp {
            x,
            y,
            op: UnconstrainedBinOpType::Eq,
        })
    }
    fn unconstrained_not_eq(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::UnconstrainedBinOp {
            x,
            y,
            op: UnconstrainedBinOpType::NotEq,
        })
    }
    fn unconstrained_bool_or(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::UnconstrainedBinOp {
            x,
            y,
            op: UnconstrainedBinOpType::BoolOr,
        })
    }
    fn unconstrained_bool_and(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::UnconstrainedBinOp {
            x,
            y,
            op: UnconstrainedBinOpType::BoolAnd,
        })
    }
    fn unconstrained_bit_or(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::UnconstrainedBinOp {
            x,
            y,
            op: UnconstrainedBinOpType::BitOr,
        })
    }
    fn unconstrained_bit_and(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::UnconstrainedBinOp {
            x,
            y,
            op: UnconstrainedBinOpType::BitAnd,
        })
    }
    fn unconstrained_bit_xor(
        &mut self,
        x: impl ToVariableOrValue<CircuitField<C>>,
        y: impl ToVariableOrValue<CircuitField<C>>,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::UnconstrainedBinOp {
            x,
            y,
            op: UnconstrainedBinOpType::BitXor,
        })
    }
}

impl<C: Config, H: HintCaller<CircuitField<C>>> RootAPI<C> for DebugBuilder<C, H> {
    fn memorized_simple_call<F: Fn(&mut Self, &Vec<Variable>) -> Vec<Variable> + 'static>(
        &mut self,
        f: F,
        inputs: &[Variable],
    ) -> Vec<Variable> {
        ensure_variables_valid(inputs);
        let inputs = inputs.to_vec();
        f(self, &inputs)
    }
    fn set_outputs(&mut self, outputs: Vec<Variable>) {
        ensure_variables_valid(&outputs);
        self.outputs = outputs;
    }
}

impl<C: Config, H: HintCaller<CircuitField<C>>> DebugBuilder<C, H> {
    pub fn new(
        inputs: Vec<CircuitField<C>>,
        public_inputs: Vec<CircuitField<C>>,
        hint_caller: H,
    ) -> (Self, Vec<Variable>, Vec<Variable>) {
        let mut builder = DebugBuilder {
            values: vec![CircuitField::<C>::zero()],
            hint_caller,
            outputs: vec![],
        };
        let vars = (1..=inputs.len()).map(new_variable).collect();
        let public_vars = (inputs.len() + 1..=inputs.len() + public_inputs.len())
            .map(new_variable)
            .collect();
        builder.values.extend(inputs);
        builder.values.extend(public_inputs);
        (builder, vars, public_vars)
    }

    fn convert_to_value<T: ToVariableOrValue<CircuitField<C>>>(&self, value: T) -> CircuitField<C> {
        match value.convert_to_variable_or_value() {
            VariableOrValue::Variable(v) => self.values[get_variable_id(v)],
            VariableOrValue::Value(v) => v,
        }
    }

    fn convert_to_id<T: ToVariableOrValue<CircuitField<C>>>(&mut self, value: T) -> usize {
        match value.convert_to_variable_or_value() {
            VariableOrValue::Variable(v) => get_variable_id(v),
            VariableOrValue::Value(v) => {
                let id = self.values.len();
                self.values.push(v);
                id
            }
        }
    }

    fn return_as_variable(&mut self, value: CircuitField<C>) -> Variable {
        let id = self.values.len();
        self.values.push(value);
        new_variable(id)
    }

    fn eval_ir_insn(&mut self, insn: IrInstruction<C>) -> Variable {
        match insn.eval_unsafe(&self.values) {
            EvalResult::Error(e) => panic!("error: {:?}", e),
            EvalResult::SubCircuitCall(_, _) => unreachable!(),
            EvalResult::Value(v) => self.return_as_variable(v),
            EvalResult::Values(_) => unreachable!(),
        }
    }

    pub fn get_outputs(&self) -> Vec<CircuitField<C>> {
        self.outputs
            .iter()
            .map(|v| self.values[get_variable_id(*v)])
            .collect()
    }
}
