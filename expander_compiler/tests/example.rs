use expander_compiler::frontend::*;
use extra::Serde;

// Circuit that asserts x^5 = y
declare_circuit!(Circuit {
    x: Variable,
    y: Variable,
});

impl Define<M31Gkr2Config> for Circuit<Variable> {
    fn define(&self, builder: &mut API<M31Gkr2Config>) {
        let x5 = builder.power_gate(self.x, 5);
        builder.assert_is_equal(x5, self.y);
    }
}

#[test]
fn example_full() {
    let compile_result = compile(&Circuit::default()).unwrap();
    let assignment = Circuit::<M31> {
        x: M31::from(2),
        y: M31::from(32),
    };
    let witness = compile_result
        .witness_solver
        .solve_witness(&assignment)
        .unwrap();
    let output = compile_result.layered_circuit.run(&witness);
    assert_eq!(output, vec![true]);

    let file = std::fs::File::create("circuit.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    compile_result
        .layered_circuit
        .serialize_into(writer)
        .unwrap();

    let file = std::fs::File::create("witness.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    witness.serialize_into(writer).unwrap();

    let file = std::fs::File::create("witness_solver.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    compile_result
        .witness_solver
        .serialize_into(writer)
        .unwrap();
}
