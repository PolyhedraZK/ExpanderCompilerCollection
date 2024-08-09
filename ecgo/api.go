// Package ecgo wraps the most commonly used compiler APIs and provides an entry point for compilation.
// This package simplifies the interaction with the compiler by exposing a unified API interface.
package ecgo

import (
	"math/big"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/builder"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/compile"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/irsource"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/irwg"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/layered"
	"github.com/consensys/gnark/frontend"
)

// API encapsulates the ecgo's frontend.API along with two new APIs added to facilitate
// direct invocation of ecgo.API within the codebase.
type API interface {
	frontend.API
	builder.SubCircuitAPI
	builder.API
}

// CompileResult represents the result of a compilation process.
// It contains unexported fields and provides methods to retrieve various components
// like the intermediate representation (IR) of the circuit, the WitnessGenerator, and the Layered Circuit.
type CompileResult struct {
	irs  *irsource.RootCircuit
	irwg *irwg.RootCircuit
	lc   *layered.RootCircuit
}

// Compile is similar to gnark's frontend.Compile. It compiles the given circuit and returns
// a pointer to CompileResult along with any error encountered during the compilation process.
func Compile(field *big.Int, circuit frontend.Circuit, opts ...frontend.CompileOption) (*CompileResult, error) {
	var root *builder.Root
	newBuilder_ := func(field *big.Int, config frontend.CompileConfig) (frontend.Builder, error) {
		if root != nil {
			panic("newBuilder can only be called once")
		}
		root = builder.NewRoot(field, config)
		return root, nil
	}
	// returned R1CS is useless
	_, err := frontend.Compile(field, newBuilder_, circuit, opts...)
	if err != nil {
		return nil, err
	}
	rc := root.Finalize()
	_ = rc
	//os.WriteFile("p1.txt", irsource.SerializeRootCircuit(rc), 0644)
	irwg, lc, err := compile.Compile(rc)
	if err != nil {
		return nil, err
	}
	return &CompileResult{irs: rc, irwg: irwg, lc: lc}, nil
}

// GetCircuitIr returns the intermediate representation (IR) of the compiled circuit as *ir.RootCircuit.
func (c *CompileResult) GetCircuitIr() *irsource.RootCircuit {
	return c.irs
}

// GetInputSolver returns the InputSolver component of the compilation result as *ir.InputSolver.
func (c *CompileResult) GetLayeredCircuit() *layered.RootCircuit {
	return c.lc
}

// GetLayeredCircuit returns the Layered Circuit component of the compilation result as *layered.RootCircuit.
func (c *CompileResult) GetInputSolver() *irwg.RootCircuit {
	return c.irwg
}

// DeserializeLayeredCircuit takes a byte buffer and returns a pointer to a layered.RootCircuit
// which represents a deserialized layered circuit.
func DeserializeLayeredCircuit(buf []byte) *layered.RootCircuit {
	return layered.DeserializeRootCircuit(buf)
}

// DeserializeInputSolver takes a byte buffer and returns a pointer to an ir.InputSolver
// which represents a deserialized input solver.
func DeserializeInputSolver(buf []byte) *irwg.RootCircuit {
	return irwg.DeserializeRootCircuit(buf)
}
