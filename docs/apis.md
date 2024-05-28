# Compiler APIs

This document provides an overview of the core APIs available in the compiler.

## Root Package

To use the compiler, include the following import in your Go code:

```go
import "github.com/PolyhedraZK/ExpanderCompilerCollection"
```

### Compile Function

`ExpanderCompilerCollection.Compile` serves as the entry point for the compiler. It accepts a `frontend.Circuit` and yields a `CompileResult`.

### CompileResult Structure

`ExpanderCompilerCollection.CompileResult` encapsulates the outcome of the compilation process, comprising both the layered circuit and the intermediate representation (IR).

The `CompileResult` offers three methods for data retrieval:

1. `GetCircuitIr`: Retrieves the IR.
2. `GetLayeredCircuit`: Obtains the layered circuit.
3. `GetInputSolver()`: Gets the compiled witness solver of the circuit.

### Builder API

`ExpanderCompilerCollection.API` is the interface for the builder API, enhancing the `frontend.API` interface from gnark with additional capabilities such as sub-circuit support and utility functions.

## Sub-Circuit API

Sub-circuit functionality is facilitated through the following methods:

```go
type SubCircuitSimpleFunc func(api frontend.API, input []frontend.Variable) []frontend.Variable
type SubCircuitFunc interface{}

type SubCircuitAPI interface {
	MemorizedSimpleCall(SubCircuitSimpleFunc, []frontend.Variable) []frontend.Variable
	MemorizedCall(SubCircuitFunc, ...interface{}) interface{}
}
```

`SubCircuitFunc` accommodates any function that accepts simple types and `frontend.Variable` as inputs and returns `frontend.Variable` as the output.

For instance, a function signature like `func(frontend.API, int, uint8, [][]frontend.Variable, string, ...[]frontend.Variable) [][][]frontend.Variable` qualifies as a `SubCircuitFunc`.

A crucial requirement for `SubCircuitFunc` is determinism: given identical simple-type inputs, the output should be consistent. This consistency is critical as the compiler memorizes the wiring of sub-circuit calls, reusing the memorized results for identical inputs.

## Additional APIs

```go
type API interface {
	ToSingleVariable(frontend.Variable) frontend.Variable
	Output(frontend.Variable)
	LayerOf(frontend.Variable) int
	ToFirstLayer(frontend.Variable) frontend.Variable
	GetRandomValue() frontend.Variable
}
```

- `ToSingleVariable`: Converts an expression into a single base variable. If the input expression is already a single variable, it is returned directly. Otherwise, an internal variable or gate is created to represent the expression.
- `Output`: Adds a variable to the circuit's output. This is used to expose certain variables as public outputs of the circuit.
- `LayerOf`: Estimates the layer in which a variable will appear in the compiled layered circuit. The term "estimate" is used because subsequent compilation optimizations may alter the exact layer placement determined during the Builder phase.
- `ToFirstLayer`: Uses a hint to pull a variable back to the first layer.
- `GetRandomValue`: Retrieves a random value directly, which is more efficient than generating pseudo-random numbers using a hash function. This is possible due to the Libra proving process which allows direct access to random numbers.

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

These function wrappers are syntactic conveniences for `MemorizedCall`. They simplify the usage of memorized calls. For example, the following invocations are functionally equivalent:

```go
output := api.MemorizedSimpleCall(f, input)

memorizedF := builder.MemorizedSimpleFunc(f)
output := memorizedF(api, input)
```