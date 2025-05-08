use arith::Field;
use arith::SimdField as _SimdField;
use expander_binary::executor;
use expander_compiler::frontend::*;
use expander_compiler::{
    declare_circuit,
    frontend::{CircuitField, Config, Define, GoldilocksConfig, RootAPI, Variable},
};
use gkr_engine::{MPIConfig, MPIEngine};
use rand::SeedableRng;
use serdes::ExpSerde;

// A fibonacci circuit that iterates for 10 times
// Note: 10 is hard coded here.
declare_circuit!(FibonacciCircuit {
    x: Variable,      // first input
    y: Variable,      // second input
    output: Variable  // output
});

impl<C: Config> Define<C> for FibonacciCircuit<Variable> {
    fn define<Builder: RootAPI<C>>(&self, api: &mut Builder) {
        let mut x = self.x;
        let mut y = self.y;
        for i in 0..10 {
            let tmp = api.add(x, y);
            x = y;
            y = api.mul(tmp,x);
            println!("i {} x: {:?}, y: {:?}", i, x, y);
            api.display("x", x);
            api.display("y", y);
        }
        api.assert_is_equal(y, self.output);
    }
}

fn circuit_gen<C: Config>(
    x: &CircuitField<C>,
    y: &CircuitField<C>,
) -> FibonacciCircuit<CircuitField<C>> {
    let mut a = x.clone();
    let mut b = y.clone();
    for i in 0..10 {
        let tmp = a + b;
        a = b;
        b = tmp * a;
        println!("i {} x: {:?}, y: {:?}", i, x, y);
    }

    FibonacciCircuit::<CircuitField<C>> {
        x: x.clone(),
        y: y.clone(),
        output: b,
    }
}

fn example<C: Config>(filename: &str) {
    let n_witnesses = SIMDField::<C>::PACK_SIZE;
    println!("n_witnesses: {}", n_witnesses);
    let compile_result: CompileResult<C> =
        compile(&FibonacciCircuit::default(), CompileOptions::default()).unwrap();
    let mut rng = rand::rngs::StdRng::seed_from_u64(1235);
    let x = CircuitField::<C>::random_unsafe(&mut rng);
    let y = CircuitField::<C>::random_unsafe(&mut rng);

    let assignment = circuit_gen::<C>(&x, &y);
    let assignments = vec![assignment; n_witnesses];
    let witness = compile_result
        .witness_solver
        .solve_witnesses(&assignments)
        .unwrap();
    let output = compile_result.layered_circuit.run(&witness);
    for x in output.iter() {
        assert!(*x);
    }

    let file = std::fs::File::create(format!("circuit_fib_{}.txt", filename)).unwrap();
    let writer = std::io::BufWriter::new(file);
    compile_result.layered_circuit.serialize_into(writer).unwrap();

    let mut expander_circuit = compile_result.layered_circuit.export_to_expander_flatten();

    let file = std::fs::File::create(format!("witness_fib_{}.txt", filename)).unwrap();
    let writer = std::io::BufWriter::new(file);
    witness.serialize_into(writer).unwrap();

    let file = std::fs::File::create(format!("witness_fib_{}_solver.txt", filename)).unwrap();
    let writer = std::io::BufWriter::new(file);
    witness.serialize_into(writer).unwrap();

    println!("dumped to files");


    let mpi_config = MPIConfig::prover_new();

    let (simd_input, simd_public_input) = witness.to_simd();
    println!("{} {}", simd_input.len(), simd_public_input.len());
    expander_circuit.layers[0].input_vals = simd_input;
    expander_circuit.public_input = simd_public_input.clone();

    // prove
    expander_circuit.evaluate();
    let (claimed_v, proof) = executor::prove::<C>(&mut expander_circuit, mpi_config.clone());

    // verify
    assert!(executor::verify::<C>(
        &mut expander_circuit,
        mpi_config,
        &proof,
        &claimed_v
    ));

  
  
}

#[test]
fn example_fib_goldilocks() {
    example::<GoldilocksConfig>("goldilocks");
    // assert!(false, "TODO: fix this test");
}
