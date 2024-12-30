mod sha256_utils;
use gf2::GF2;
use rand::RngCore;
use sha256_utils::add_vanilla;

use expander_compiler::{declare_circuit, frontend::*};

declare_circuit!(VanillaAdder {
    a: [Variable; 32],
    b: [Variable; 32],
    c: [Variable; 32],
});

impl<C: Config> Define<C> for VanillaAdder<Variable> {
    fn define(&self, api: &mut API<C>) {
        let c_target = add_vanilla(api, self.a.to_vec(), self.b.to_vec());
        for i in 0..32 {
            api.assert_is_equal(self.c[i], c_target[i]);
        }
    }
}

impl<C: Config> GenericDefine<C> for VanillaAdder<Variable> {
    fn define<Builder: RootAPI<C>>(&self, builder: &mut Builder) {
        let c_target = add_vanilla(builder, self.a.to_vec(), self.b.to_vec());
        for i in 0..32 {
            builder.assert_is_equal(self.c[i], c_target[i]);
        }
    }
}

#[test]
fn test_add_vanilla() {
    let mut rng = rand::thread_rng();

    let n_tests = 100;
    let mut assignments = vec![];
    for _ in 0..n_tests {
        let a = rng.next_u32();
        let b = rng.next_u32();
        let (c, _overflowed) = a.overflowing_add(b);

        let mut assignment = VanillaAdder::<GF2>::default();
        for i in 0..32 {
            assignment.a[i] = ((a >> i) & 1).into();
            assignment.b[i] = ((b >> i) & 1).into();
            assignment.c[i] = ((c >> i) & 1).into();
        }

        assignments.push(assignment);
    }

    // layered circuit
    let compile_result: CompileResult<GF2Config> = compile(&VanillaAdder::default()).unwrap();
    let CompileResult {
        witness_solver,
        layered_circuit,
    } = compile_result;
    let witness = witness_solver.solve_witnesses(&assignments).unwrap();
    let res = layered_circuit.run(&witness);
    let expected_res = vec![true; n_tests];
    assert_eq!(res, expected_res);

    // crosslayer circuit
    let compile_result: CompileResultCrossLayer<GF2Config> =
        compile_generic_cross_layer(&VanillaAdder::default(), CompileOptions::default()).unwrap();
    let CompileResultCrossLayer::<GF2Config> {
        witness_solver,
        layered_circuit,
    } = compile_result;
    let witness = witness_solver.solve_witnesses(&assignments).unwrap();
    let res = layered_circuit.run(&witness);
    let expected_res = vec![true; n_tests];
    assert_eq!(res, expected_res);
}
