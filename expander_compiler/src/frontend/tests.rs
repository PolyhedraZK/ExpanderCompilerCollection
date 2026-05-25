use crate::frontend::M31Config as C;
use crate::{
    compile::CompileOptions,
    field::{FieldArith, M31},
    frontend::{compile, BasicAPI, RootAPI},
};

use super::{builder::Variable, circuit::*, variables::DumpLoadTwoVariables};

declare_circuit!(Circuit1 {
    a: Variable,
    b: [PublicVariable; 2],
    c: u64,
    d: [[Variable; 3]; 5],
    e: [[[u64; 4]; 3]; 2],
});

#[test]
fn test_circuit_declaration() {
    use crate::field::M31 as F;
    let c = Circuit1::<F> {
        a: F::one(),
        b: [F::one(), F::zero()],
        c: 1,
        d: [
            [F::one(), F::zero(), F::one()],
            [F::zero(), F::one(), F::zero()],
            [F::one(), F::zero(), F::one()],
            [F::zero(), F::one(), F::zero()],
            [F::one(), F::zero(), F::one()],
        ],
        e: [
            [[1, 2, 3, 4], [5, 6, 7, 8], [9, 10, 11, 12]],
            [[13, 14, 15, 16], [17, 18, 19, 20], [21, 22, 23, 24]],
        ],
    };
    assert_eq!(c.num_vars(), (1 + 3 * 5, 2));
    let mut vars = vec![];
    let mut public_vars = vec![];
    c.dump_into(&mut vars, &mut public_vars);
    assert_eq!((vars.len(), public_vars.len()), c.num_vars());
    let mut c2 = Circuit1::<F>::default();
    let vars_ref = &mut vars.as_slice();
    let public_vars_ref = &mut public_vars.as_slice();
    c2.load_from(vars_ref, public_vars_ref);
    assert_eq!(vars_ref.len(), 0);
    assert_eq!(public_vars_ref.len(), 0);
    assert_eq!(c.a, c2.a);
    assert_eq!(c.b, c2.b);
    assert_eq!(c.d, c2.d);
}

declare_circuit!(Circuit2 {
    sum: Variable,
    x: [Variable; 2],
});

impl Define<C> for Circuit2<Variable> {
    fn define<Builder: RootAPI<C>>(&self, builder: &mut Builder) {
        let sum = builder.add(self.x[0], self.x[1]);
        let sum = builder.add(sum, 123);
        builder.assert_is_equal(sum, self.sum);
    }
}

#[test]
fn test_circuit_eval_simple() {
    let compile_result = compile(&Circuit2::default(), CompileOptions::default()).unwrap();
    let assignment = Circuit2::<M31> {
        sum: M31::from(126 as u32),
        x: [M31::from(1 as u32), M31::from(2 as u32)],
    };
    let witness = compile_result
        .witness_solver
        .solve_witness(&assignment)
        .unwrap();
    let output = compile_result.layered_circuit.run(&witness);
    assert_eq!(output, vec![true]);

    let assignment = Circuit2::<M31> {
        sum: M31::from(127 as u32),
        x: [M31::from(1 as u32), M31::from(2 as u32)],
    };
    let witness = compile_result
        .witness_solver
        .solve_witness(&assignment)
        .unwrap();
    let output = compile_result.layered_circuit.run(&witness);
    assert_eq!(output, vec![false]);
}

// linear_combination tests:
// verify that linear_combination([(x,a),(y,b)], c) == c + a*x + b*y via circuit eval

declare_circuit!(LinCombCircuit {
    out_lc: Variable,  // computed via linear_combination
    out_ref: Variable, // computed via repeated mul+add (reference)
    x: Variable,
    y: Variable,
});

impl Define<C> for LinCombCircuit<Variable> {
    fn define<Builder: RootAPI<C>>(&self, builder: &mut Builder) {
        use crate::field::M31;
        let a = M31::from(3u32);
        let b = M31::from(5u32);
        let c = M31::from(7u32);

        let lc = builder.linear_combination(&[(self.x, a), (self.y, b)], c);
        builder.assert_is_equal(lc, self.out_lc);

        // reference: c + a*x + b*y via repeated mul+add
        let ax = builder.mul(self.x, a);
        let by = builder.mul(self.y, b);
        let axby = builder.add(ax, by);
        let ref_val = builder.add(axby, c);
        builder.assert_is_equal(ref_val, self.out_ref);
    }
}

#[test]
fn test_linear_combination_matches_mul_add() {
    let compile_result = compile(&LinCombCircuit::default(), CompileOptions::default()).unwrap();

    // x=2, y=4 → lc = 7 + 3*2 + 5*4 = 7+6+20 = 33
    let assignment = LinCombCircuit::<M31> {
        out_lc: M31::from(33u32),
        out_ref: M31::from(33u32),
        x: M31::from(2u32),
        y: M31::from(4u32),
    };
    let witness = compile_result
        .witness_solver
        .solve_witness(&assignment)
        .unwrap();
    let output = compile_result.layered_circuit.run(&witness);
    assert_eq!(output, vec![true]);
}

declare_circuit!(LinCombZeroCoefCircuit {
    out: Variable,
    x: Variable,
    y: Variable,
});

impl Define<C> for LinCombZeroCoefCircuit<Variable> {
    fn define<Builder: RootAPI<C>>(&self, builder: &mut Builder) {
        use crate::field::M31;
        // zero coefficient on y; linear_combination must ignore it
        let a = M31::from(3u32);
        let zero = M31::from(0u32);
        let c = M31::from(1u32);
        let lc = builder.linear_combination(&[(self.x, a), (self.y, zero)], c);
        builder.assert_is_equal(lc, self.out);
    }
}

#[test]
fn test_linear_combination_zero_coef_ignored() {
    let compile_result = compile(
        &LinCombZeroCoefCircuit::default(),
        CompileOptions::default(),
    )
    .unwrap();

    // y has zero coef → out = 1 + 3*x regardless of y
    let assignment = LinCombZeroCoefCircuit::<M31> {
        out: M31::from(7u32), // 1 + 3*2 = 7
        x: M31::from(2u32),
        y: M31::from(999u32), // ignored
    };
    let witness = compile_result
        .witness_solver
        .solve_witness(&assignment)
        .unwrap();
    let output = compile_result.layered_circuit.run(&witness);
    assert_eq!(output, vec![true]);
}

declare_circuit!(LinCombAllZeroCircuit {
    out: Variable,
    x: Variable,
});

impl Define<C> for LinCombAllZeroCircuit<Variable> {
    fn define<Builder: RootAPI<C>>(&self, builder: &mut Builder) {
        use crate::field::M31;
        // all zero coefs → falls back to pure constant
        let zero = M31::from(0u32);
        let c = M31::from(42u32);
        let lc = builder.linear_combination(&[(self.x, zero)], c);
        builder.assert_is_equal(lc, self.out);
    }
}

#[test]
fn test_linear_combination_all_zero_coefs_returns_constant() {
    let compile_result =
        compile(&LinCombAllZeroCircuit::default(), CompileOptions::default()).unwrap();

    let assignment = LinCombAllZeroCircuit::<M31> {
        out: M31::from(42u32),
        x: M31::from(5u32), // irrelevant
    };
    let witness = compile_result
        .witness_solver
        .solve_witness(&assignment)
        .unwrap();
    let output = compile_result.layered_circuit.run(&witness);
    assert_eq!(output, vec![true]);
}

#[test]
#[should_panic]
fn test_linear_combination_invalid_variable_panics() {
    use crate::frontend::builder::Builder;
    let (mut b, _inputs) = Builder::<C>::new(2);
    let bad = Variable::default(); // id=0, invalid
    let c = M31::from(0u32);
    // must panic due to ensure_variable_valid
    let _ = b.linear_combination(&[(bad, M31::from(1u32))], c);
}
