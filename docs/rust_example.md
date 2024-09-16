# Rust Example

Below is an example using the `expander_compiler` library in Rust:

```rust
use expander_compiler::frontend::*;
use internal::Serde;

declare_circuit!(Circuit {
    x: Variable,
    y: Variable,
});

impl Define<M31Config> for Circuit<Variable> {
    fn define(&self, builder: &mut API<M31Config>) {
        builder.assert_is_equal(self.x, self.y);
    }
}

#[test]
fn example_full() {
    let compile_result = compile(&Circuit::default()).unwrap();
    let assignment = Circuit::<M31> {
        x: M31::from(123),
        y: M31::from(123),
    };
    let witness = compile_result
        .witness_solver
        .solve_witness(&assignment)
        .unwrap();
    let output = compile_result.layered_circuit.run(&witness);
    assert_eq!(output, vec![true]);

    // Serialize and write the circuit to a file
    let file = std::fs::File::create("circuit.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    compile_result
        .layered_circuit
        .serialize_into(writer)
        .unwrap();

    // Serialize and write the witness to a file
    let file = std::fs::File::create("witness.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    witness.serialize_into(writer).unwrap();

    // Serialize and write the witness solver to a file
    let file = std::fs::File::create("witness_solver.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    compile_result
        .witness_solver
        .serialize_into(writer)
        .unwrap();
}
```

In this example, we demonstrate how to define a simple circuit, compile it using the `expander_compiler` library, solve the witness, run and verify the circuit, and then serialize the circuit, witness, and witness solver to files.

## Detailed Explanation

1. **Define the Circuit**:
    ```rust
    declare_circuit!(Circuit {
        x: Variable,
        y: Variable,
    });
    ```

2. **Implement the Circuit Logic**:
    ```rust
    impl Define<M31Config> for Circuit<Variable> {
        fn define(&self, builder: &mut API<M31Config>) {
            builder.assert_is_equal(self.x, self.y);
        }
    }
    ```

3. **Compile and Solve the Witness**:
    ```rust
    let compile_result = compile(&Circuit::default()).unwrap();
    let assignment = Circuit::<M31> {
        x: M31::from(123),
        y: M31::from(123),
    };
    let witness = compile_result
        .witness_solver
        .solve_witness(&assignment)
        .unwrap();
    ```

4. **Run and Verify the Circuit**:
    ```rust
    let output = compile_result.layered_circuit.run(&witness);
    assert_eq!(output, vec![true]);
    ```

5. **Serialize and Write to Files**:
    ```rust
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
    ```

By following these steps, we can define, compile, and verify a circuit, and serialize the relevant data for later use.

This example can also be found in [this file](../expander_compiler/tests/example.rs).