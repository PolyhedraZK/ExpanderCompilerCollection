//! This module contains the expression used in the IR.

use std::{
    fmt,
    io::{Read, Write},
    ops::{Deref, DerefMut},
};

use serdes::{ExpSerde, SerdeResult};

use crate::circuit::config::{CircuitField, Config};
use crate::field::FieldArith;

/// The `Term` struct represents a term in an expression, which consists of a coefficient and a variable specification.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Term<C: Config> {
    pub coef: CircuitField<C>,
    pub vars: VarSpec,
}

/// The `VarSpec` enum represents the specification of variables in a term.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum VarSpec {
    /// Represents a constant term.
    Const,
    /// Represents a linear term with a single variable.
    Linear(usize),
    /// Represents a quadratic term with two variables.
    Quad(usize, usize),
    /// Represents a custom gate term with a specific gate type and inputs.
    Custom {
        gate_type: usize,
        inputs: Vec<usize>,
    },
    /// Represents a random linear term, where the coefficient will be ignored.
    RandomLinear(usize),
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
            VarSpec::Custom { .. } => {}
            VarSpec::RandomLinear(_) => {}
        }
    }
    fn is_normalized(&self) -> bool {
        match self {
            VarSpec::Const => true,
            VarSpec::Linear(_) => true,
            VarSpec::Quad(index1, index2) => index1 >= index2,
            VarSpec::Custom { .. } => true,
            VarSpec::RandomLinear(_) => true,
        }
    }
    /// Multiplies two `VarSpec` instances and returns the resulting `VarSpec`.
    /// If the multiplication is invalid (e.g., multiplying a linear term with a quadratic term), it panics.
    pub fn mul(a: &Self, b: &Self) -> Self {
        match (a, b) {
            (VarSpec::Const, VarSpec::Const) => VarSpec::Const,
            (VarSpec::Const, VarSpec::Linear(x)) => VarSpec::Linear(*x),
            (VarSpec::Const, VarSpec::Quad(x, y)) => VarSpec::Quad(*x, *y),
            (VarSpec::Const, VarSpec::Custom { gate_type, inputs }) => VarSpec::Custom {
                gate_type: *gate_type,
                inputs: inputs.clone(),
            },
            (VarSpec::Linear(x), VarSpec::Const) => VarSpec::Linear(*x),
            (VarSpec::Linear(x), VarSpec::Linear(y)) => VarSpec::Quad(*x, *y),
            (VarSpec::Linear(_), VarSpec::Quad(_, _)) => panic!("invalid multiplication"),
            (VarSpec::Linear(_), VarSpec::Custom { .. }) => panic!("invalid multiplication"),
            (VarSpec::Quad(x, y), VarSpec::Const) => VarSpec::Quad(*x, *y),
            (VarSpec::Quad(_, _), VarSpec::Linear(_)) => panic!("invalid multiplication"),
            (VarSpec::Quad(_, _), VarSpec::Quad(_, _)) => panic!("invalid multiplication"),
            (VarSpec::Quad(_, _), VarSpec::Custom { .. }) => panic!("invalid multiplication"),
            (VarSpec::Custom { gate_type, inputs }, VarSpec::Const) => VarSpec::Custom {
                gate_type: *gate_type,
                inputs: inputs.clone(),
            },
            (VarSpec::Custom { .. }, VarSpec::Linear(_)) => panic!("invalid multiplication"),
            (VarSpec::Custom { .. }, VarSpec::Quad(_, _)) => panic!("invalid multiplication"),
            (VarSpec::Custom { .. }, VarSpec::Custom { .. }) => panic!("invalid multiplication"),
            (VarSpec::RandomLinear(_), _) => panic!("unexpected situation: RandomLinear"),
            (_, VarSpec::RandomLinear(_)) => panic!("unexpected situation: RandomLinear"),
        }
    }
    /// Replaces the variable indices in the `VarSpec` with new indices according to the provided function.
    pub fn replace_vars<F: Fn(usize) -> usize>(&self, f: F) -> Self {
        match self {
            VarSpec::Const => VarSpec::Const,
            VarSpec::Linear(x) => VarSpec::Linear(f(*x)),
            VarSpec::Quad(x, y) => VarSpec::Quad(f(*x), f(*y)),
            VarSpec::Custom { gate_type, inputs } => VarSpec::Custom {
                gate_type: *gate_type,
                inputs: inputs.iter().cloned().map(&f).collect(),
            },
            VarSpec::RandomLinear(x) => VarSpec::RandomLinear(f(*x)),
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
    /// Creates a new constant term with the given value.
    pub fn new_const(value: CircuitField<C>) -> Self {
        Term {
            coef: value,
            vars: VarSpec::Const,
        }
    }
    /// Creates a new linear term with the given value and variable index.
    pub fn new_linear(value: CircuitField<C>, index: usize) -> Self {
        Term {
            coef: value,
            vars: VarSpec::Linear(index),
        }
    }
    /// Creates a new quadratic term with the given value and variable indices.
    pub fn new_quad(value: CircuitField<C>, index1: usize, index2: usize) -> Self {
        Term {
            coef: value,
            vars: if index1 < index2 {
                VarSpec::Quad(index2, index1)
            } else {
                VarSpec::Quad(index1, index2)
            },
        }
    }
    /// Creates a new random linear term with the given index.
    pub fn new_random_linear(index: usize) -> Self {
        Term {
            coef: CircuitField::<C>::one(),
            vars: VarSpec::RandomLinear(index),
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
        Term::new_const(CircuitField::<C>::zero())
    }
}

impl<C: Config> Term<C> {
    /// Multiplies two terms and returns the resulting term.
    /// If the multiplication is invalid (e.g., multiplying a linear term with a quadratic term), it panics.
    pub fn mul(&self, other: &Self) -> Self {
        Term {
            coef: self.coef * other.coef,
            vars: VarSpec::mul(&self.vars, &other.vars),
        }
    }
}

impl<C: Config> fmt::Display for Term<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.coef == CircuitField::<C>::one() {
            match &self.vars {
                VarSpec::Const => write!(f, "1"),
                VarSpec::Linear(index) => write!(f, "v{index}"),
                VarSpec::Quad(index1, index2) => write!(f, "v{index1}*v{index2}"),
                VarSpec::Custom { gate_type, inputs } => {
                    write!(f, "custom{gate_type}(")?;
                    for (i, input) in inputs.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",")?;
                        }
                        write!(f, "v{input}")?;
                    }
                    write!(f, ")")
                }
                VarSpec::RandomLinear(index) => write!(f, "random*v{index}"),
            }
        } else {
            match &self.vars {
                VarSpec::Const => write!(f, "{}", self.coef.to_u256()),
                VarSpec::Linear(index) => write!(f, "v{}*{}", index, self.coef.to_u256()),
                VarSpec::Quad(index1, index2) => {
                    write!(f, "v{}*v{}*{}", index1, index2, self.coef.to_u256())
                }
                VarSpec::Custom { gate_type, inputs } => {
                    write!(f, "custom{gate_type}(")?;
                    for (i, input) in inputs.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",")?;
                        }
                        write!(f, "v{input}")?;
                    }
                    write!(f, ")*{}", self.coef.to_u256())
                }
                VarSpec::RandomLinear(_) => panic!("unexpected situation: RandomLinear"),
            }
        }
    }
}

/// The `Expression` struct represents a mathematical expression consisting of multiple terms.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

impl<C: Config> fmt::Display for Expression<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, term) in self.terms.iter().enumerate() {
            if i > 0 {
                write!(f, " + ")?;
            }
            write!(f, "{term}")?;
        }
        Ok(())
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
    if terms.is_empty() {
        terms.push(Term::default());
    }
}

impl<C: Config> Expression<C> {
    /// Creates a new expression with a single constant term.
    pub fn new_const(value: CircuitField<C>) -> Self {
        Expression {
            terms: vec![Term::new_const(value)],
        }
    }
    /// Creates a new expression with a single linear term.
    pub fn new_linear(value: CircuitField<C>, index: usize) -> Self {
        Expression {
            terms: vec![Term::new_linear(value, index)],
        }
    }
    /// Creates a new expression with a single quadratic term.
    pub fn new_quad(value: CircuitField<C>, index1: usize, index2: usize) -> Self {
        Expression {
            terms: vec![Term::new_quad(value, index1, index2)],
        }
    }
    /// Creates a new expression with a single custom term.
    pub fn new_custom(value: CircuitField<C>, gate_type: usize, inputs: Vec<usize>) -> Self {
        Expression {
            terms: vec![Term {
                coef: value,
                vars: VarSpec::Custom { gate_type, inputs },
            }],
        }
    }
    /// Creates a new expression from a list of terms, normalizing each term and sorting them.
    pub fn from_terms(mut terms: Vec<Term<C>>) -> Self {
        for term in terms.iter_mut() {
            term.normalize();
        }
        terms.sort();
        compress_identical_terms(&mut terms);
        Expression { terms }
    }
    /// Creates a new expression from a list of terms, ensuring they are sorted and normalized.
    /// If it's not sorted, it's undefined behavior.
    pub fn from_terms_sorted(mut terms: Vec<Term<C>>) -> Self {
        if terms.is_empty() {
            terms.push(Term::default());
        }
        for term in terms.iter() {
            assert!(term.is_normalized());
        }
        assert!(terms.windows(2).all(|w| w[0].vars < w[1].vars));
        Expression { terms }
    }
    /// Creates an empty expression, which is considered invalid.
    pub fn invalid() -> Self {
        Expression { terms: vec![] }
    }
    /// Get variable indices from the expression.
    pub fn get_vars<R: std::iter::FromIterator<usize>>(&self) -> R {
        self.iter()
            .flat_map(|term| match &term.vars {
                VarSpec::Const => vec![],
                VarSpec::Linear(index) => vec![*index],
                VarSpec::Quad(index1, index2) => vec![*index1, *index2],
                VarSpec::Custom { inputs, .. } => inputs.clone(),
                VarSpec::RandomLinear(index) => vec![*index],
            })
            .collect()
    }
    /// Replaces variable indices in the expression according to the provided function.
    pub fn replace_vars<F: Fn(usize) -> usize>(&self, f: F) -> Self {
        let terms = self
            .iter()
            .map(|term| Term {
                coef: term.coef,
                vars: term.vars.replace_vars(&f),
            })
            .collect();
        Expression { terms }
    }
    /// Returns the degree of the expression.
    pub fn degree(&self) -> usize {
        let mut has_linear = false;
        for term in self.iter() {
            match term.vars {
                VarSpec::Const => {}
                VarSpec::Linear(_) => has_linear = true,
                VarSpec::Quad(_, _) => return 2,
                VarSpec::Custom { .. } => return 2,
                VarSpec::RandomLinear(_) => panic!("unexpected situation: RandomLinear"),
            }
        }
        if has_linear {
            1
        } else {
            0
        }
    }
    /// Returns the count of terms with different degrees in the expression.
    pub fn count_of_degrees(&self) -> [usize; 3] {
        let mut res = [0; 3];
        for term in self.iter() {
            match term.vars {
                VarSpec::Const => res[0] += 1,
                VarSpec::Linear(_) => res[1] += 1,
                VarSpec::Quad(_, _) => res[2] += 1,
                VarSpec::Custom { .. } => res[2] += 1,
                VarSpec::RandomLinear(_) => panic!("unexpected situation: RandomLinear"),
            }
        }
        res
    }
    /// Returns the constant value of the expression if it consists of a single constant term.
    pub fn constant_value(&self) -> Option<CircuitField<C>> {
        if self.terms.len() == 1 && self.terms[0].vars == VarSpec::Const {
            Some(self.terms[0].coef)
        } else {
            None
        }
    }
    /// Multiplies the expression by a constant value, returning a new expression.
    pub fn mul_constant(&self, value: CircuitField<C>) -> Self {
        if value.is_zero() {
            return Expression::default();
        }
        Expression::from_terms_sorted(
            self.iter()
                .map(|term| Term {
                    coef: term.coef * value,
                    vars: term.vars.clone(),
                })
                .collect(),
        )
    }
    /// Converts the expression to a vector of terms.
    pub fn to_terms(self) -> Vec<Term<C>> {
        self.terms
    }
}

/// Linear combination term, which consists of a variable index and a coefficient.
#[derive(Debug, Clone, Hash, PartialEq, Eq, ExpSerde)]
pub struct LinCombTerm<C: Config> {
    /// The variable index in the circuit.
    pub var: usize,
    /// The coefficient of the term.
    pub coef: CircuitField<C>,
}

/// A linear combination, which is a sum of terms with coefficients and a constant.
/// It is used to represent linear expressions in the circuit,
/// especially in early stages of compilation where expressions are not yet normalized.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct LinComb<C: Config> {
    /// The terms in the linear combination.
    pub terms: Vec<LinCombTerm<C>>,
    /// The constant term in the linear combination.
    pub constant: CircuitField<C>,
}

impl<C: Config> Default for LinComb<C> {
    fn default() -> Self {
        LinComb {
            terms: vec![],
            constant: CircuitField::<C>::zero(),
        }
    }
}

impl<C: Config> LinComb<C> {
    /// Gets the variable indices from the linear combination.
    pub fn get_vars(&self) -> Vec<usize> {
        self.terms.iter().map(|term| term.var).collect()
    }
    /// Replaces the variable indices in the linear combination according to the provided function.
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
    /// Creates a linear combination representing the expression `kx + b`, where `x` is a variable index, `k` is a coefficient, and `b` is a constant.
    pub fn from_kx_plus_b(x: usize, k: CircuitField<C>, b: CircuitField<C>) -> Self {
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
    /// Evaluates the linear combination using the provided values for the variables.
    pub fn eval(&self, values: &[CircuitField<C>]) -> CircuitField<C> {
        let mut res = self.constant;
        for term in self.terms.iter() {
            res += values[term.var] * term.coef;
        }
        res
    }
    /// Evaluates the linear combination using SIMD values for the variables.
    pub fn eval_simd<SF: arith::SimdField<Scalar = CircuitField<C>>>(&self, values: &[SF]) -> SF {
        let mut res = SF::one().scale(&self.constant);
        for term in self.terms.iter() {
            res += values[term.var].scale(&term.coef);
        }
        res
    }
}

impl<C: Config> fmt::Display for LinComb<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, term) in self.terms.iter().enumerate() {
            if i > 0 {
                write!(f, " + ")?;
            }
            if term.coef == CircuitField::<C>::one() {
                write!(f, "v{}", term.var)?;
            } else {
                write!(f, "v{}*{}", term.var, term.coef.to_u256())?;
            }
        }
        if !self.constant.is_zero() {
            write!(f, " + {}", self.constant.to_u256())?;
        }
        Ok(())
    }
}

impl<C: Config> ExpSerde for LinComb<C> {
    fn serialize_into<W: Write>(&self, mut writer: W) -> SerdeResult<()> {
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

    fn deserialize_from<R: Read>(mut reader: R) -> SerdeResult<Self> {
        let len = usize::deserialize_from(&mut reader)?;
        let mut terms = Vec::with_capacity(len);
        for _ in 0..len {
            let var = usize::deserialize_from(&mut reader)?;
            terms.push(LinCombTerm {
                var,
                coef: CircuitField::<C>::zero(),
            });
        }
        for term in terms.iter_mut() {
            term.coef = CircuitField::<C>::deserialize_from(&mut reader)?;
        }
        let constant = CircuitField::<C>::deserialize_from(&mut reader)?;
        Ok(LinComb { terms, constant })
    }
}
