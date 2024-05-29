# Compilation Process

The compilation process, encapsulated within `api.go`, comprises several distinct phases:

1. **Circuit Intermediate Representation (IR) Construction:** The initial phase involves processing the source code with the builder, which constructs the circuit's Intermediate Representation (IR). This builder, similar to gnark's internal builder, extends the `LinearExpression` to support quadratic terms. It maintains a tree structure that mirrors the computational graph of the circuit, serving as the foundation for IR generation.

2. **IR Optimization:** The Intermediate Representation (IR) is a flexible representation of the circuit that allows for various optimizations. These can include merging redundant variables, splitting large variables to enhance efficiency, and performing other transformations that streamline the computational graph without altering its functionality.

3. **Circuit Layering:** The IR undergoes a layering process that transforms it into a layered circuit. This process involves topologically sorting the directed acyclic graph (DAG) of sub-circuit call relationships, determining the necessary layers for each variable, and allocating the variables across these layers. The layering strategy is recursive, giving priority to sub-circuits and ensuring alignment with the Libra protocol's requirements.

4. **Layered Circuit Optimization:** After the circuit is layered, additional optimizations can be applied. These may include expanding sub-circuits based on their frequency of occurrence or the number of variables they contain.

## Detailed Steps in Compilation

### Builder Implementation

The builder captures the high-level algorithmic structure and translates it into a form suitable for optimization and layering. It records the overall tree structure, then generates the IR, a more flexible form for subsequent compilation steps.

### Intermediate Representation (IR)

The Intermediate Representation (IR) plays a crucial role in the compilation process, acting as a conduit between the high-level circuit description and the layered circuit prepared for proof generation. The IR comprises instructions, which can manifest as variables, hints, or sub-circuit calls.

### Layering Process

The layering process involves the following steps:

1. Topologically sort the Directed Acyclic Graph (DAG) formed by all circuits (sub-circuit call graphs).
2. Calculate the layers where each variable needs to appear for each circuit.
3. Allocate the layout of variables for each layer.
4. Compile the connections between all adjacent layers into `layered/Circuit`.

Additional details include:

1. Assertions are divided by layer and accumulated up to the output layer.
2. If a sub-circuit uses a hint, these hints must be relayed from the input layer.

This process effectively eliminates most unnecessary variables and redundant sub-circuit calls, optimizing the overall performance.

### Layered Circuit

For detailed information on the format of the layered circuit, please refer to the [artifact and serialization](./artifact_and_serialization.md) documentation.

The first element in the circuit's output layer is the Random Combined Assertions, followed by Public Inputs.

Current optimizations include the expansion of sub-circuits that have few occurrences or variables, further enhancing the efficiency of the system.