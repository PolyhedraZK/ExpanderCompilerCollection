use crate::{
    circuit::{
        config::Config,
        ir::{
            common::{EvalResult, Instruction},
            source::{BoolBinOpType, Instruction as IrInstruction, UnconstrainedBinOpType},
        },
    },
    field::FieldArith,
};

use super::{
    api::{BasicAPI, DebugAPI, RootAPI, UnconstrainedAPI},
    builder::{get_variable_id, new_variable, ToVariableOrValue, VariableOrValue},
    Variable,
};

pub struct DebugBuilder<C: Config> {
    values: Vec<C::CircuitField>,
}

impl<C: Config> BasicAPI<C> for DebugBuilder<C> {
    fn add(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) -> Variable {
        let x = self.convert_to_value(x);
        let y = self.convert_to_value(y);
        self.return_as_variable(x + y)
    }
    fn sub(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) -> Variable {
        let x = self.convert_to_value(x);
        let y = self.convert_to_value(y);
        self.return_as_variable(x - y)
    }
    fn mul(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) -> Variable {
        let x = self.convert_to_value(x);
        let y = self.convert_to_value(y);
        self.return_as_variable(x * y)
    }
    fn xor(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
        checked: bool,
    ) -> Variable {
        let x = self.convert_to_id(x);
        let y = self.convert_to_id(y);
        self.eval_ir_insn(IrInstruction::Div { x, y, checked })
    }
    fn neg(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable {
        let x = self.convert_to_value(x);
        self.return_as_variable(-x)
    }
    fn is_zero(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable {
        let x = self.convert_to_id(x);
        self.eval_ir_insn(IrInstruction::IsZero(x))
    }
    fn assert_is_zero(&mut self, x: impl ToVariableOrValue<C::CircuitField>) {
        let x = self.convert_to_value(x);
        assert!(x.is_zero());
    }
    fn assert_is_non_zero(&mut self, x: impl ToVariableOrValue<C::CircuitField>) {
        let x = self.convert_to_value(x);
        assert!(!x.is_zero());
    }
    fn assert_is_bool(&mut self, x: impl ToVariableOrValue<C::CircuitField>) {
        let x = self.convert_to_value(x);
        assert!(x.is_zero() || x == C::CircuitField::one());
    }
    fn get_random_value(&mut self) -> Variable {
        let v = C::CircuitField::random_unsafe(&mut rand::thread_rng());
        self.return_as_variable(v)
    }
    fn constant(&mut self, x: impl ToVariableOrValue<<C as Config>::CircuitField>) -> Variable {
        let x = self.convert_to_value(x);
        self.return_as_variable(x)
    }
}

impl<C: Config> UnconstrainedAPI<C> for DebugBuilder<C> {
    fn unconstrained_identity(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable {
        self.constant(x)
    }
    fn unconstrained_add(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) -> Variable {
        self.add(x, y)
    }
    fn unconstrained_mul(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
    ) -> Variable {
        self.mul(x, y)
    }
    fn unconstrained_div(
        &mut self,
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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
        x: impl ToVariableOrValue<C::CircuitField>,
        y: impl ToVariableOrValue<C::CircuitField>,
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

impl<C: Config> DebugAPI<C> for DebugBuilder<C> {
    fn value_of(&self, x: impl ToVariableOrValue<C::CircuitField>) -> C::CircuitField {
        self.convert_to_value(x)
    }
}

impl<C: Config> RootAPI<C> for DebugBuilder<C> {
    fn memorized_simple_call<F: Fn(&mut Self, &Vec<Variable>) -> Vec<Variable> + 'static>(
        &mut self,
        f: F,
        inputs: &[Variable],
    ) -> Vec<Variable> {
        let inputs = inputs.to_vec();
        f(self, &inputs)
    }
}

impl<C: Config> DebugBuilder<C> {
    pub fn new(
        inputs: Vec<C::CircuitField>,
        public_inputs: Vec<C::CircuitField>,
    ) -> (Self, Vec<Variable>, Vec<Variable>) {
        let mut builder = DebugBuilder {
            values: vec![C::CircuitField::zero()],
        };
        let vars = (1..=inputs.len()).map(new_variable).collect();
        let public_vars = (inputs.len() + 1..=inputs.len() + public_inputs.len())
            .map(new_variable)
            .collect();
        builder.values.extend(inputs);
        builder.values.extend(public_inputs);
        (builder, vars, public_vars)
    }

    fn convert_to_value<T: ToVariableOrValue<C::CircuitField>>(&self, value: T) -> C::CircuitField {
        match value.convert_to_variable_or_value() {
            VariableOrValue::Variable(v) => self.values[get_variable_id(v)],
            VariableOrValue::Value(v) => v,
        }
    }

    fn convert_to_id<T: ToVariableOrValue<C::CircuitField>>(&mut self, value: T) -> usize {
        match value.convert_to_variable_or_value() {
            VariableOrValue::Variable(v) => get_variable_id(v),
            VariableOrValue::Value(v) => {
                let id = self.values.len();
                self.values.push(v);
                id
            }
        }
    }

    fn return_as_variable(&mut self, value: C::CircuitField) -> Variable {
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
}