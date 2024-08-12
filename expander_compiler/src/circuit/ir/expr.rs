use std::{
    io::{Error as IoError, Read, Write},
    ops::{Deref, DerefMut},
};

use crate::circuit::config::Config;
use crate::field::Field;
use crate::utils::serde::Serde;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Term<C: Config> {
    pub coef: C::CircuitField,
    pub vars: VarSpec,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum VarSpec {
    Const,
    Linear(usize),
    Quad(usize, usize),
}

impl VarSpec {
    fn normalize(&mut self) {
        match self {
            VarSpec::Const => {}
            VarSpec::Linear(_) => {}
            VarSpec::Quad(index1, index2) => {
                if index1 < index2 {
                    *self = VarSpec::Quad(*index2, *index1);
                }
            }
        }
    }
    fn is_normalized(&self) -> bool {
        match self {
            VarSpec::Const => true,
            VarSpec::Linear(_) => true,
            VarSpec::Quad(index1, index2) => index1 >= index2,
        }
    }
    pub fn mul(a: &Self, b: &Self) -> Self {
        match (a, b) {
            (VarSpec::Const, VarSpec::Const) => VarSpec::Const,
            (VarSpec::Const, VarSpec::Linear(x)) => VarSpec::Linear(*x),
            (VarSpec::Const, VarSpec::Quad(x, y)) => VarSpec::Quad(*x, *y),
            (VarSpec::Linear(x), VarSpec::Const) => VarSpec::Linear(*x),
            (VarSpec::Linear(x), VarSpec::Linear(y)) => VarSpec::Quad(*x, *y),
            (VarSpec::Linear(_), VarSpec::Quad(_, _)) => panic!("invalid multiplication"),
            (VarSpec::Quad(x, y), VarSpec::Const) => VarSpec::Quad(*x, *y),
            (VarSpec::Quad(_, _), VarSpec::Linear(_)) => panic!("invalid multiplication"),
            (VarSpec::Quad(_, _), VarSpec::Quad(_, _)) => panic!("invalid multiplication"),
        }
    }
}

impl<C: Config> Ord for Term<C> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let res = self.vars.cmp(&other.vars);
        if res == std::cmp::Ordering::Equal {
            self.coef.cmp(&other.coef)
        } else {
            res
        }
    }
}

impl<C: Config> PartialOrd for Term<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<C: Config> Term<C> {
    pub fn new_const(value: C::CircuitField) -> Self {
        Term {
            coef: value,
            vars: VarSpec::Const,
        }
    }
    pub fn new_linear(value: C::CircuitField, index: usize) -> Self {
        Term {
            coef: value,
            vars: VarSpec::Linear(index),
        }
    }
    pub fn new_quad(value: C::CircuitField, index1: usize, index2: usize) -> Self {
        Term {
            coef: value,
            vars: if index1 < index2 {
                VarSpec::Quad(index2, index1)
            } else {
                VarSpec::Quad(index1, index2)
            },
        }
    }
    fn normalize(&mut self) {
        self.vars.normalize();
    }
    fn is_normalized(&self) -> bool {
        self.vars.is_normalized()
    }
}

impl<C: Config> Default for Term<C> {
    fn default() -> Self {
        Term::new_const(C::CircuitField::zero())
    }
}

impl<C: Config> Term<C> {
    pub fn mul(&self, other: &Self) -> Self {
        Term {
            coef: self.coef * other.coef,
            vars: VarSpec::mul(&self.vars, &other.vars),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Expression<C: Config> {
    terms: Vec<Term<C>>,
}

impl<C: Config> Deref for Expression<C> {
    type Target = Vec<Term<C>>;
    fn deref(&self) -> &Self::Target {
        &self.terms
    }
}

impl<C: Config> DerefMut for Expression<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terms
    }
}

impl<C: Config> Default for Expression<C> {
    fn default() -> Self {
        Expression {
            terms: vec![Term::default()],
        }
    }
}

// requires terms to be sorted
fn compress_identical_terms<C: Config>(terms: &mut Vec<Term<C>>) {
    let mut i = 0;
    for j in 1..terms.len() {
        if terms[i].vars == terms[j].vars {
            let j_coef = terms[j].coef;
            terms[i].coef += j_coef;
        } else {
            i += 1;
            terms[i] = terms[j].clone();
        }
    }
    terms.truncate(i + 1);
    terms.retain(|term| !term.coef.is_zero());
    if terms.len() == 0 {
        terms.push(Term::default());
    }
}

impl<C: Config> Expression<C> {
    pub fn new_const(value: C::CircuitField) -> Self {
        Expression {
            terms: vec![Term::new_const(value)],
        }
    }
    pub fn new_linear(value: C::CircuitField, index: usize) -> Self {
        Expression {
            terms: vec![Term::new_linear(value, index)],
        }
    }
    pub fn new_quad(value: C::CircuitField, index1: usize, index2: usize) -> Self {
        Expression {
            terms: vec![Term::new_quad(value, index1, index2)],
        }
    }
    pub fn from_terms(mut terms: Vec<Term<C>>) -> Self {
        for term in terms.iter_mut() {
            term.normalize();
        }
        terms.sort();
        compress_identical_terms(&mut terms);
        Expression { terms }
    }
    pub fn from_terms_sorted(mut terms: Vec<Term<C>>) -> Self {
        if terms.len() == 0 {
            terms.push(Term::default());
        }
        for term in terms.iter() {
            assert!(term.is_normalized());
        }
        assert!(terms.windows(2).all(|w| w[0].vars < w[1].vars));
        Expression { terms }
    }
    pub fn invalid() -> Self {
        Expression { terms: vec![] }
    }
    pub fn get_vars<R: std::iter::FromIterator<usize>>(&self) -> R {
        self.iter()
            .flat_map(|term| match term.vars {
                VarSpec::Const => vec![],
                VarSpec::Linear(index) => vec![index],
                VarSpec::Quad(index1, index2) => vec![index1, index2],
            })
            .collect()
    }
    pub fn replace_vars<F: Fn(usize) -> usize>(&self, f: F) -> Self {
        let terms = self
            .iter()
            .map(|term| Term {
                coef: term.coef,
                vars: match term.vars {
                    VarSpec::Const => VarSpec::Const,
                    VarSpec::Linear(index) => VarSpec::Linear(f(index)),
                    VarSpec::Quad(index1, index2) => VarSpec::Quad(f(index1), f(index2)),
                },
            })
            .collect();
        Expression { terms }
    }
    pub fn degree(&self) -> usize {
        let mut has_linear = false;
        for term in self.iter() {
            match term.vars {
                VarSpec::Const => {}
                VarSpec::Linear(_) => has_linear = true,
                VarSpec::Quad(_, _) => return 2,
            }
        }
        if has_linear {
            1
        } else {
            0
        }
    }
    pub fn constant_value(&self) -> Option<C::CircuitField> {
        if self.terms.len() == 1 && self.terms[0].vars == VarSpec::Const {
            Some(self.terms[0].coef)
        } else {
            None
        }
    }
    pub fn mul_constant(&self, value: C::CircuitField) -> Self {
        if value.is_zero() {
            return Expression::default();
        }
        Expression::from_terms_sorted(
            self.iter()
                .map(|term| Term {
                    coef: term.coef * value,
                    vars: term.vars,
                })
                .collect(),
        )
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct LinCombTerm<C: Config> {
    pub var: usize,
    pub coef: C::CircuitField,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct LinComb<C: Config> {
    pub terms: Vec<LinCombTerm<C>>,
    pub constant: C::CircuitField,
}

impl<C: Config> Default for LinComb<C> {
    fn default() -> Self {
        LinComb {
            terms: vec![],
            constant: C::CircuitField::zero(),
        }
    }
}

impl<C: Config> LinComb<C> {
    pub fn get_vars(&self) -> Vec<usize> {
        self.terms.iter().map(|term| term.var).collect()
    }
    pub fn replace_vars<F: Fn(usize) -> usize>(&self, f: F) -> Self {
        LinComb {
            terms: self
                .terms
                .iter()
                .map(|term| LinCombTerm {
                    var: f(term.var),
                    coef: term.coef,
                })
                .collect(),
            constant: self.constant,
        }
    }
    pub fn from_kx_plus_b(x: usize, k: C::CircuitField, b: C::CircuitField) -> Self {
        if x == 0 || k.is_zero() {
            LinComb {
                terms: vec![],
                constant: b,
            }
        } else {
            LinComb {
                terms: vec![LinCombTerm { var: x, coef: k }],
                constant: b,
            }
        }
    }
    pub fn eval(&self, values: &[C::CircuitField]) -> C::CircuitField {
        let mut res = self.constant;
        for term in self.terms.iter() {
            res += values[term.var] * term.coef;
        }
        res
    }
}

impl<C: Config> Serde for LinComb<C> {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        self.terms.len().serialize_into(&mut writer)?;
        for term in self.terms.iter() {
            term.var.serialize_into(&mut writer)?;
        }
        for term in self.terms.iter() {
            term.coef.serialize_into(&mut writer)?;
        }
        self.constant.serialize_into(&mut writer)?;
        Ok(())
    }
    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let len = usize::deserialize_from(&mut reader)?;
        let mut terms = Vec::with_capacity(len);
        for _ in 0..len {
            let var = usize::deserialize_from(&mut reader)?;
            terms.push(LinCombTerm {
                var,
                coef: C::CircuitField::zero(),
            });
        }
        for term in terms.iter_mut() {
            term.coef = C::CircuitField::deserialize_from(&mut reader)?;
        }
        let constant = C::CircuitField::deserialize_from(&mut reader)?;
        Ok(LinComb { terms, constant })
    }
}
