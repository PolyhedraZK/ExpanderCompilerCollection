use arith::Field;
use expander_compiler::frontend::*;
use gkr::executor::{BN254ConfigMIMC5, GF2ExtConfigSha2, M31ExtConfigSha2};

declare_circuit!(Circuit {
    s: [Variable; 100],
    sum: PublicVariable
});

impl<C: Config> Define<C> for Circuit<Variable> {
    fn define(&self, builder: &mut API<C>) {
        let mut sum = builder.constant(0);
        for x in self.s.iter() {
            sum = builder.add(sum, x);
        }
        builder.assert_is_equal(sum, self.sum);
    }
}

fn example<C: Config, GKRFieldC, GKRC>()
where
    GKRFieldC: gkr_field_config::GKRFieldConfig<CircuitField = C::CircuitField>,
    GKRC: expander_config::GKRConfig<FieldConfig = GKRFieldC>,
{
    let n_witnesses = <GKRFieldC::SimdCircuitField as arith::SimdField>::PACK_SIZE;
    println!("n_witnesses: {}", n_witnesses);
    let compile_result: CompileResult<C> = compile(&Circuit::default()).unwrap();
    let mut s = [C::CircuitField::zero(); 100];
    for i in 0..s.len() {
        s[i] = C::CircuitField::random_unsafe(&mut rand::thread_rng());
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
        assert_eq!(*x, true);
    }

    let mut expander_circuit = compile_result
        .layered_circuit
        .export_to_expander::<GKRFieldC>()
        .flatten();
    let config = expander_config::Config::<GKRC>::new(
        expander_config::GKRScheme::Vanilla,
        mpi_config::MPIConfig::new(),
    );

    let (simd_input, simd_public_input) = witness.to_simd::<GKRFieldC::SimdCircuitField>();
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
    example::<GF2Config, gkr_field_config::GF2ExtConfig, GF2ExtConfigSha2>();
}

#[test]
fn example_m31() {
    example::<M31Config, gkr_field_config::M31ExtConfig, M31ExtConfigSha2>();
}

#[test]
fn example_bn254() {
    example::<BN254Config, gkr_field_config::BN254Config, BN254ConfigMIMC5>();
}
