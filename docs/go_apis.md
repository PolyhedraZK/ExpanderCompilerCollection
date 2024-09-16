# Compiler APIs

This document offers a comprehensive guide to the core APIs provided by the compiler.

## Root Package

To integrate the compiler into your Go project, add the following import statement to your code:

```go
import "github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo"
```

### Compile Function

The `ecgo.Compile` method is the primary interface to the compiler. It takes a `frontend.Circuit` as input and returns a `CompileResult`.

### CompileResult Structure

The `ecgo.CompileResult` structure encapsulates the results of the compilation process, which includes both the layered circuit and the intermediate representation (IR).

The `CompileResult` provides three methods for accessing the data:

1. `GetCircuitIr`: This method retrieves the IR.
2. `GetLayeredCircuit`: This method fetches the layered circuit.
3. `GetInputSolver()`: This method obtains the compiled witness solver of the circuit.

### Builder API

The `ecgo.API` serves as the interface for the builder API. It extends the `frontend.API` interface from gnark, offering additional features such as support for sub-circuits and utility functions.

## Sub-Circuit API

The functionality of sub-circuits is enabled via the following methods:

```go
type SubCircuitSimpleFunc func(api frontend.API, input []frontend.Variable) []frontend.Variable
type SubCircuitFunc interface{}

type SubCircuitAPI interface {
	MemorizedSimpleCall(SubCircuitSimpleFunc, []frontend.Variable) []frontend.Variable
	MemorizedCall(SubCircuitFunc, ...interface{}) interface{}
}
```

`SubCircuitFunc` is designed to accommodate any function that takes simple types and `frontend.Variable` as inputs and returns `frontend.Variable` as the output.

For example, a function with the signature `func(frontend.API, int, uint8, [][]frontend.Variable, string, ...[]frontend.Variable) [][][]frontend.Variable` would be considered a `SubCircuitFunc`.

One key requirement for `SubCircuitFunc` is determinism. This means that for any given set of identical simple-type inputs, the output must always be the same. This is crucial because the compiler memorizes the wiring of sub-circuit calls and reuses these memorized results for identical inputs, enhancing efficiency.

## Additional APIs

```go
type API interface {
	Output(frontend.Variable)
	GetRandomValue() frontend.Variable
	CustomGate(gateType uint64, inputs ...frontend.Variable) frontend.Variable
}
```

- `Output`: This method appends a variable to the circuit's output. It is typically used to designate certain variables as public outputs of the circuit.
- `GetRandomValue`: This method retrieves a random value directly, a more efficient approach than generating pseudo-random numbers using a hash function. This direct access to random numbers is facilitated by the Libra proving process.
- `CustomGate`: This method is similar to Gnark's `NewHint` in that it essentially calls a hint function to compute a result. In the resulting layered circuit, it will be compiled into a custom gate of the specified gate type. Unlike `NewHint`, it requires pre-registering the hint function and other parameters. For specific details, see [the example](../ecgo/examples/custom_gate/main.go).

Several other APIs exist for old pure Golang Expander Compiler compability, but they are no-op now.

## Builder Extensions

### Memorized Function Wrappers

```go
func MemorizedSimpleFunc(f SubCircuitSimpleFunc) SubCircuitSimpleFunc
func MemorizedVoidFunc(f SubCircuitFunc) func(frontend.API, ...interface{})
func Memorized0DFunc(f SubCircuitFunc) func(frontend.API, ...interface{}) frontend.Variable
func Memorized1DFunc(f SubCircuitFunc) func(frontend.API, ...interface{}) []frontend.Variable
func Memorized2DFunc(f SubCircuitFunc) func(frontend.API, ...interface{}) [][]frontend.Variable
func Memorized3DFunc(f SubCircuitFunc) func(frontend.API, ...interface{}) [][][]frontend.Variable
```

These function wrappers serve as syntactic convenience for `MemorizedCall`, streamlining its usage. For instance, the invocations below are functionally identical:

```go
output := api.MemorizedSimpleCall(f, input)

memorizedF := builder.MemorizedSimpleFunc(f)
output := memorizedF(api, input)
```