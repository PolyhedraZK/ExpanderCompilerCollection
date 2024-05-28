# Compiler Process

The compilation process is encapsulated within `api.go` and involves the following phases:

1. **Circuit IR Construction:** The source code is first processed by the builder, which constructs the circuit intermediate representation (IR). This builder is akin to the gnark's internal builder but extends the `LinearExpression` to support quadratic terms. It maintains a tree structure that represents the computational graph of the circuit and from which the IR is generated.

2. **IR Optimization:** The IR, as a representation of the circuit, is subject to optimization. This may involve merging redundant variables, splitting large variables to improve efficiency, and other transformations that streamline the computational graph without altering its functionality.

3. **Circuit Layering:** The IR is then subjected to a layering process that transforms it into a layered circuit. This involves topologically sorting the directed acyclic graph (DAG) of sub-circuit call relationships, computing the necessary layers for each variable, and allocating the variables across these layers. The layering strategy is recursive, prioritizing sub-circuits and ensuring alignment with the Libra protocol's requirements.

4. **Layered Circuit Optimization:** Once the circuit is layered, further optimizations can be applied to the layered circuit. These optimizations may involve expanding sub-circuits based on their frequency of occurrence or the number of variables involved.

## Detailed Steps in Compilation

### Builder Implementation

The builder is responsible for capturing the high-level algorithmic structure and translating it into a form suitable for optimization and layering. It records the overall tree structure, then generates the IR, which is a more malleable form for subsequent compilation steps.

### IR (Intermediate Representation)

The IR is a pivotal stage in the compilation process, serving as a bridge between the high-level description of the circuit and the layered circuit ready for use in proofs. The IR consists of instructions that can be variables, hints, or sub-circuit calls.

### Layering Process

1. Topologically sorts the DAG formed by all circuits (sub-circuit call graphs).
2. For each circuit, calculates the layers where each variable needs to appear.
3. Allocates the layout of variables for each layer.
4. Compiles the connections between all adjacent layers into `layered/Circuit`.

Details include:
1. Assertions are divided by layer and accumulated up to the output layer.
2. If a sub-circuit uses a hint, these hints must be relayed from the input layer.

This process eliminates most unnecessary variables and redundant sub-circuit calls.

### Layered Circuit

Refer to [Layered Circuit Format](./layered_circuit_format.md) for details on the layered circuit's format.

The first element in the circuit's output layer is the Random Combined Assertions, followed by Public Inputs.

Current optimizations include the expansion of sub-circuits that have few occurrences or variables.