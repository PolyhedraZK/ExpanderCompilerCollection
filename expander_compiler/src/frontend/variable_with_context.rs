use std::{
    any::Any,
    cell::RefCell,
    ops::{Add, Mul, Neg, Sub},
};

use crate::circuit::config::Config;

use super::{
    builder::{Builder, RawVariable},
    variable::Variable,
};

thread_local! {
    static BUILDERS:RefCell<Vec<Box< dyn Any>>> = RefCell::new(Vec::new());
}

fn push_builder<C: Config>(builder: Builder<C>) {
    BUILDERS.with(|builders| {
        builders.borrow_mut().push(Box::new(builder));
    });
}

fn pop_builder<C: Config>() -> Builder<C> {
    BUILDERS.with(|builders| {
        let builder = builders.borrow_mut().pop().unwrap();
        *builder.downcast().unwrap()
    })
}

fn eval_on_builder<C: Config, F: FnOnce(&mut Builder<C>) -> T, T>(f: F) -> T {
    BUILDERS.with(|builders| {
        let mut builders = builders.borrow_mut();
        let builder = builders.last_mut().unwrap();
        let builder = builder.downcast_mut().unwrap();
        f(builder)
    })
}

pub fn eval_with_builder<C: Config, T, F: FnOnce() -> T>(
    builder: Builder<C>,
    f: F,
) -> (T, Builder<C>) {
    push_builder(builder);
    let res = f();
    (res, pop_builder())
}

#[derive(Clone, Copy)]
struct VariableWithContext<C: Config> {
    variable: RawVariable,
    _a: C,
}

impl<C: Config> Variable for VariableWithContext<C> {}

impl<C: Config> Add for VariableWithContext<C> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let variable =
            eval_on_builder(|builder: &mut Builder<C>| builder.add(self.variable, rhs.variable));
        VariableWithContext {
            variable,
            _a: self._a,
        }
    }
}

impl<C: Config> Sub for VariableWithContext<C> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        let variable =
            eval_on_builder(|builder: &mut Builder<C>| builder.sub(self.variable, rhs.variable));
        VariableWithContext {
            variable,
            _a: self._a,
        }
    }
}

impl<C: Config> Mul for VariableWithContext<C> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let variable =
            eval_on_builder(|builder: &mut Builder<C>| builder.mul(self.variable, rhs.variable));
        VariableWithContext {
            variable,
            _a: self._a,
        }
    }
}

impl<C: Config> Neg for VariableWithContext<C> {
    type Output = Self;
    fn neg(self) -> Self {
        let variable = eval_on_builder(|builder: &mut Builder<C>| builder.neg(self.variable));
        VariableWithContext {
            variable,
            _a: self._a,
        }
    }
}
