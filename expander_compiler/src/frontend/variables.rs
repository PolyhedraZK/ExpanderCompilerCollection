//! This module provides traits and implementations for dumping and loading variables in circuits.

use super::builder::Variable;
use crate::field::Field;

/// This trait defines methods for dumping and loading variables in a circuit.
///
/// This trait should be automatically implemented for circuit structs.
pub trait DumpLoadVariables<T: Sized + Clone> {
    /// Dumps the variable into a vector of variables.
    fn dump_into(&self, vars: &mut Vec<T>);
    /// Loads the variable from a slice of variables.
    fn load_from(&mut self, vars: &mut &[T]);
    /// Returns the number of variables this type represents.
    fn num_vars(&self) -> usize;
}

pub trait DumpLoadTwoVariables<T: Sized + Clone> {
    fn dump_into(&self, vars: &mut Vec<T>, public_vars: &mut Vec<T>);
    fn load_from(&mut self, vars: &mut &[T], public_vars: &mut &[T]);
    fn num_vars(&self) -> (usize, usize);
}

impl<F: Field> DumpLoadVariables<F> for F {
    fn dump_into(&self, vars: &mut Vec<F>) {
        vars.push(*self);
    }
    fn load_from(&mut self, vars: &mut &[F]) {
        *self = vars[0];
        *vars = &vars[1..];
    }
    fn num_vars(&self) -> usize {
        1
    }
}

impl DumpLoadVariables<Variable> for Variable {
    fn dump_into(&self, vars: &mut Vec<Variable>) {
        vars.push(*self);
    }
    fn load_from(&mut self, vars: &mut &[Variable]) {
        *self = vars[0];
        *vars = &vars[1..];
    }
    fn num_vars(&self) -> usize {
        1
    }
}

impl<T: Clone, U, const N: usize> DumpLoadVariables<T> for [U; N]
where
    U: DumpLoadVariables<T>,
{
    fn dump_into(&self, vars: &mut Vec<T>) {
        for x in self.iter() {
            x.dump_into(vars);
        }
    }
    fn load_from(&mut self, vars: &mut &[T]) {
        for x in self.iter_mut() {
            x.load_from(vars);
        }
    }
    fn num_vars(&self) -> usize {
        N * self[0].num_vars()
    }
}
