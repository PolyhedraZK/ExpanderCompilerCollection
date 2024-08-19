use crate::{
    circuit::{
        config::Config,
        ir::{
            expr::{LinComb, LinCombTerm},
            source::{self, Constraint as SourceConstraint, Instruction as SourceInstruction},
        },
        layered::Coef,
    },
    field::{Field, U256},
};

pub struct Builder<C: Config> {
    instructions: Vec<SourceInstruction<C>>,
    constraints: Vec<SourceConstraint>,
    var_max: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct RawVariable {
    id: usize,
}

pub enum VariableOrValue<F: Field> {
    Variable(RawVariable),
    Value(F),
}

pub trait ToVariableOrValue<F: Field> {
    fn to_variable_or_value(self) -> VariableOrValue<F>;
}

trait NotVariable {}
impl NotVariable for u32 {}
impl NotVariable for U256 {}

impl<F: Field, T: Into<F> + NotVariable> ToVariableOrValue<F> for T {
    fn to_variable_or_value(self) -> VariableOrValue<F> {
        VariableOrValue::Value(self.into())
    }
}

impl<F: Field> ToVariableOrValue<F> for RawVariable {
    fn to_variable_or_value(self) -> VariableOrValue<F> {
        VariableOrValue::Variable(self)
    }
}

impl<F: Field> ToVariableOrValue<F> for &RawVariable {
    fn to_variable_or_value(self) -> VariableOrValue<F> {
        VariableOrValue::Variable(*self)
    }
}

impl<C: Config> Builder<C> {
    pub fn to_variable<T: ToVariableOrValue<C::CircuitField>>(&mut self, value: T) -> RawVariable {
        match value.to_variable_or_value() {
            VariableOrValue::Variable(v) => v,
            VariableOrValue::Value(v) => {
                self.instructions
                    .push(SourceInstruction::ConstantOrRandom(Coef::Constant(v)));
                self.var_max += 1;
                RawVariable { id: self.var_max }
            }
        }
    }

    pub fn new_var(&mut self) -> RawVariable {
        self.var_max += 1;
        RawVariable { id: self.var_max }
    }

    pub fn add<T: ToVariableOrValue<C::CircuitField>, U: ToVariableOrValue<C::CircuitField>>(
        &mut self,
        x: T,
        y: U,
    ) -> RawVariable {
        let x = self.to_variable(x);
        let y = self.to_variable(y);
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

    pub fn sub<T: ToVariableOrValue<C::CircuitField>, U: ToVariableOrValue<C::CircuitField>>(
        &mut self,
        x: T,
        y: U,
    ) -> RawVariable {
        let x = self.to_variable(x);
        let y = self.to_variable(y);
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

    pub fn neg<T: ToVariableOrValue<C::CircuitField>>(&mut self, x: T) -> RawVariable {
        let x = self.to_variable(x);
        self.instructions.push(SourceInstruction::LinComb(LinComb {
            terms: vec![LinCombTerm {
                var: x.id,
                coef: -C::CircuitField::one(),
            }],
            constant: C::CircuitField::zero(),
        }));
        self.new_var()
    }

    pub fn mul<T: ToVariableOrValue<C::CircuitField>, U: ToVariableOrValue<C::CircuitField>>(
        &mut self,
        x: T,
        y: U,
    ) -> RawVariable {
        let x = self.to_variable(x);
        let y = self.to_variable(y);
        self.instructions
            .push(SourceInstruction::Mul(vec![x.id, y.id]));
        self.new_var()
    }

    pub fn div<T: ToVariableOrValue<C::CircuitField>, U: ToVariableOrValue<C::CircuitField>>(
        &mut self,
        x: T,
        y: U,
        checked: bool,
    ) -> RawVariable {
        let x = self.to_variable(x);
        let y = self.to_variable(y);
        self.instructions.push(SourceInstruction::Div {
            x: x.id,
            y: y.id,
            checked: checked,
        });
        self.new_var()
    }

    pub fn inverse<T: ToVariableOrValue<C::CircuitField>>(&mut self, x: T) -> RawVariable {
        self.div(1, x, true)
    }

    pub fn xor<T: ToVariableOrValue<C::CircuitField>, U: ToVariableOrValue<C::CircuitField>>(
        &mut self,
        x: T,
        y: U,
    ) -> RawVariable {
        let x = self.to_variable(x);
        let y = self.to_variable(y);
        self.instructions.push(SourceInstruction::BoolBinOp {
            x: x.id,
            y: y.id,
            op: source::BoolBinOpType::Xor,
        });
        self.new_var()
    }

    pub fn or<T: ToVariableOrValue<C::CircuitField>, U: ToVariableOrValue<C::CircuitField>>(
        &mut self,
        x: T,
        y: U,
    ) -> RawVariable {
        let x = self.to_variable(x);
        let y = self.to_variable(y);
        self.instructions.push(SourceInstruction::BoolBinOp {
            x: x.id,
            y: y.id,
            op: source::BoolBinOpType::Or,
        });
        self.new_var()
    }

    pub fn and<T: ToVariableOrValue<C::CircuitField>, U: ToVariableOrValue<C::CircuitField>>(
        &mut self,
        x: T,
        y: U,
    ) -> RawVariable {
        let x = self.to_variable(x);
        let y = self.to_variable(y);
        self.instructions.push(SourceInstruction::BoolBinOp {
            x: x.id,
            y: y.id,
            op: source::BoolBinOpType::And,
        });
        self.new_var()
    }

    pub fn is_zero<T: ToVariableOrValue<C::CircuitField>>(&mut self, x: T) -> RawVariable {
        let x = self.to_variable(x);
        self.instructions.push(SourceInstruction::IsZero(x.id));
        self.new_var()
    }

    pub fn assert_is_zero<T: ToVariableOrValue<C::CircuitField>>(&mut self, x: T) {
        let x = self.to_variable(x);
        self.constraints.push(SourceConstraint {
            typ: source::ConstraintType::Zero,
            var: x.id,
        });
    }

    pub fn assert_is_non_zero<T: ToVariableOrValue<C::CircuitField>>(&mut self, x: T) {
        let x = self.to_variable(x);
        self.constraints.push(SourceConstraint {
            typ: source::ConstraintType::NonZero,
            var: x.id,
        });
    }

    pub fn assert_is_bool<T: ToVariableOrValue<C::CircuitField>>(&mut self, x: T) {
        let x = self.to_variable(x);
        self.constraints.push(SourceConstraint {
            typ: source::ConstraintType::Bool,
            var: x.id,
        });
    }
}
