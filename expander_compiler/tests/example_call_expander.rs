use arith::Field;
use expander_compiler::frontend::*;
use rand::SeedableRng;

declare_circuit!(Circuit {
    s: [Variable; 100],
    sum: PublicVariable
});

impl<C: Config> Define<C> for Circuit<Variable> {
    fn define<Builder: RootAPI<C>>(&self, api: &mut Builder) {
        let mut sum = api.constant(0);
        for x in self.s.iter() {
            sum = api.add(sum, x);
        }
        api.assert_is_equal(sum, self.sum);
    }
}

fn example<C: Config>() {
    let n_witnesses = <C::DefaultSimdField as arith::SimdField>::PACK_SIZE;
    println!("n_witnesses: {}", n_witnesses);
    let compile_result: CompileResult<C> =
        compile_generic(&Circuit::default(), CompileOptions::default()).unwrap();
    let mut s = [C::CircuitField::zero(); 100];
    let mut rng = rand::rngs::StdRng::seed_from_u64(1235);
    for i in 0..s.len() {
        s[i] = C::CircuitField::random_unsafe(&mut rng);
    }
    let assignment = Circuit::<C::CircuitField> {
        s,
        sum: s.iter().sum(),
    };
    let assignments = vec![assignment; n_witnesses];
    let witness = compile_result
        .witness_solver
        .solve_witnesses(&assignments)
        .unwrap();
    let output = compile_result.layered_circuit.run(&witness);
    for x in output.iter() {
        assert!(*x);
    }

    let mut expander_circuit = compile_result
        .layered_circuit
        .export_to_expander::<C::DefaultGKRFieldConfig>()
        .flatten();
    let config = expander_config::Config::<C::DefaultGKRConfig>::new(
        expander_config::GKRScheme::Vanilla,
        mpi_config::MPIConfig::new(),
    );

    let (simd_input, simd_public_input) = witness.to_simd::<C::DefaultSimdField>();
    println!("{} {}", simd_input.len(), simd_public_input.len());
    expander_circuit.layers[0].input_vals = simd_input;
    expander_circuit.public_input = simd_public_input.clone();

    // prove
    expander_circuit.evaluate();
    let (claimed_v, proof) = gkr::executor::prove(&mut expander_circuit, &config);

    // verify
    assert!(gkr::executor::verify(
        &mut expander_circuit,
        &config,
        &proof,
        &claimed_v
    ));
}

#[test]
fn example_gf2() {
    example::<GF2Config>();
}

#[test]
fn example_m31() {
    example::<M31Config>();
}

#[test]
fn example_bn254() {
    example::<BN254Config>();
}
