# APIs

This page introduces core APIs of the compiler.

## Root package

```go
import gkr "github.com/Zklib/gkr-compiler"
```

### Compile

`gkr.Compile` is the main function of the compiler. It takes a `frontend.Circuit` and returns a `CompileResult`.

### CompileResult

`gkr.CompileResult` is the result of the compilation. It contains the layered circuit and the IR.

It provides 3 methods to access them:

1. `GetCircuitIr`: returns the IR.
2. `GetLayeredCircuit`: returns the layered circuit.
3. `GetWitness`: given an assignment, calculates the witness.

### API

`gkr.API` is the API interface of the builder, which extends the `frontend.API` interface of gnark and adds sub-circuit support.

The sub-circuit support is provided by the following methods:

```go
type SubCircuitSimpleFunc func(api frontend.API, input []frontend.Variable) []frontend.Variable
type SubCircuitFunc interface{}

type SubCircuitAPI interface {
	MemorizedSimpleCall(SubCircuitSimpleFunc, []frontend.Variable) []frontend.Variable
	MemorizedCall(SubCircuitFunc, ...interface{}) interface{}
}
```

`SubCircuitFunc` can be arbitrary function, which takes simple types and `frontend.Variable` as input, and returns `frontend.Variable` as output.

For example, `func(frontend.API, int, uint8, [][]frontend.Variable, string, ...[]frontend.Variable) [][][]frontend.Variable` is a valid `SubCircuitFunc`.

`SubCircuitFunc` should follow another rule: it should be deterministic if those simple type inputs are the same. This is because the compiler will memorize the circuit wiring of the sub-circuit calls, and use the memorized result if the same input is given.

## builder

### MemorizeXXXFunc

```go
func MemorizedSimpleFunc(f SubCircuitSimpleFunc) SubCircuitSimpleFunc
func MemorizedVoidFunc(f SubCircuitFunc) func(frontend.API, ...interface{})
func Memorized0DFunc(f SubCircuitFunc) func(frontend.API, ...interface{}) frontend.Variable
func Memorized1DFunc(f SubCircuitFunc) func(frontend.API, ...interface{}) []frontend.Variable
func Memorized2DFunc(f SubCircuitFunc) func(frontend.API, ...interface{}) [][]frontend.Variable
func Memorized3DFunc(f SubCircuitFunc) func(frontend.API, ...interface{}) [][][]frontend.Variable
```

These are just sugars for `MemorizedCall`. For example, the following two are equivalent:

```go
output := api.MemorizedSimpleCall(f, input)

memorizedF := builder.MemorizedSimpleFunc(f)
output := memorizedF(api, input)
```