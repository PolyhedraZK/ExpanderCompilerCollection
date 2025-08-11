use expander_compiler::{circuit::layered::witness::WitnessValues, frontend::*};
use serdes::ExpSerde;

fn inner_product(a: &[M31], b: &mut [M31]) -> Result<(), Error> {
    let mut sum = M31::ZERO;
    let n = a.len() / 2;
    for i in 0..n {
        let t = a[i] * a[i + n];
        sum += t;
    }
    b[0] = sum;
    Ok(())
}

declare_circuit!(Circuit {
    x: [Variable; 10],
    // y: [Variable; 10],
    z: Variable,
});

impl Define<M31Config> for Circuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let y = builder.custom_gate(12348, &self.x);
        let mut sum = builder.mul(&self.x[0], &self.x[5]);
        for i in 1..5 {
            let s = builder.mul(&self.x[i], &self.x[i + 5]);
            sum = builder.add(sum, s);
        }
        builder.assert_is_equal(y, sum);
        // builder.assert_is_equal(self.z, y);
        // builder.assert_is_equal(self.z, sum);
    }
}

#[test]
fn test_inner_product() {
    let mut registry = HintRegistry::<M31>::new();
    registry.register("inner_product", inner_product);
    registry.register_custom_gate(12348, "inner_product");

    let compile_result = compile(&Circuit::default(), CompileOptions::default()).unwrap();
    let assignment = Circuit::<M31> {
        x: [M31::from(2); 10],
        z: M31::from(20),
    };
    let witness = compile_result
        .witness_solver
        .solve_witness_with_hints(&assignment, &registry)
        .unwrap();
if let WitnessValues::Scalar(values) = witness.clone().values {
    println!("witness {:?}", values);
}
    let output = compile_result.layered_circuit.run_with_options(&witness, false, &registry).0;
    assert_eq!(output, vec![true]);

    // Serialize and write the circuit to a file
    let file = std::fs::File::create("custom_gate_circuit.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    compile_result
        .layered_circuit
        .serialize_into(writer)
        .unwrap();

    // Serialize and write the witness to a file
    let file = std::fs::File::create("custom_gate_witness.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    witness.serialize_into(writer).unwrap();

    // Serialize and write the witness solver to a file
    let file = std::fs::File::create("custom_gate_witness_solver.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    compile_result
        .witness_solver
        .serialize_into(writer)
        .unwrap();
}