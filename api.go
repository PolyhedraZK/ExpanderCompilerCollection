// Package gkr wraps the most commonly used compiler APIs and provides an entry point for compilation.
// This package simplifies the interaction with the compiler by exposing a unified API interface.
package gkr

import (
	"math/big"

	"github.com/Zklib/gkr-compiler/builder"
	"github.com/Zklib/gkr-compiler/ir"
	"github.com/Zklib/gkr-compiler/layered"
	"github.com/Zklib/gkr-compiler/layering"
	"github.com/Zklib/gkr-compiler/utils"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/logger"
)

// API encapsulates the gkr's frontend.API along with two new APIs added to facilitate
// direct invocation of gkr.API within the codebase.
type API interface {
	frontend.API
	builder.SubCircuitAPI
	builder.API
}

// CompileResult represents the result of a compilation process.
// It contains unexported fields and provides methods to retrieve various components
// like the intermediate representation (IR) of the circuit, the InputSolver, and the Layered Circuit.
type CompileResult struct {
	rc         *ir.RootCircuit
	compiled   *layered.RootCircuit
	inputOrder *ir.InputOrder
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
	log := logger.Logger()
	rc := root.Finalize()
	if err := ir.Validate(rc); err != nil {
		return nil, err
	}
	stats := rc.GetStats()
	log.Info().
		Int("nbRootInput", stats.NbRootInput).
		Int("nbTotTerms", stats.NbTotTerms).
		Int("nbExpandedTerms", stats.NbExpandedTerms).
		Int("nbConstraints", stats.NbConstraints).
		Msg("built circuit ir")
	// There should be some optimizations, but it requires more work to make them correct
	// rc = ir.Optimize(rc)
	rc = ir.AdjustForLayering(rc)
	if err := ir.ValidateForLayering(rc); err != nil {
		return nil, err
	}
	stats = rc.GetStats()
	log.Info().
		Int("nbRootInput", stats.NbRootInput).
		Int("nbTotTerms", stats.NbTotTerms).
		Int("nbExpandedTerms", stats.NbExpandedTerms).
		Int("nbConstraints", stats.NbConstraints).
		Msg("optimized and adjusted circuit ir")
	lrc, io := layering.Compile(rc)
	if err := layered.Validate(lrc); err != nil {
		return nil, err
	}
	if err := layered.ValidateInitialized(lrc); err != nil {
		return nil, err
	}
	lstats := lrc.GetStats()
	log.Info().
		Int("nbLayer", lstats.NbLayer).
		Int("nbCircuit", lstats.NbCircuit).
		Int("nbTotMul", lstats.NbTotMul).
		Int("nbTotAdd", lstats.NbTotAdd).
		Int("nbTotCst", lstats.NbTotCst).
		Int("nbExpandedMul", lstats.NbExpandedMul).
		Int("nbExpandedAdd", lstats.NbExpandedAdd).
		Int("nbExpandedCst", lstats.NbExpandedCst).
		Int("nbVariables", lstats.NbTotGates).
		Int("nbUsedVariables", lstats.NbUsedGates).
		Msg("compiled layered circuit")
	lrc = layered.Optimize(lrc)
	if err := layered.Validate(lrc); err != nil {
		return nil, err
	}
	if err := layered.ValidateInitialized(lrc); err != nil {
		return nil, err
	}
	lstats = lrc.GetStats()
	log.Info().
		Int("nbLayer", lstats.NbLayer).
		Int("nbCircuit", lstats.NbCircuit).
		Int("nbTotMul", lstats.NbTotMul).
		Int("nbTotAdd", lstats.NbTotAdd).
		Int("nbTotCst", lstats.NbTotCst).
		Int("nbExpandedMul", lstats.NbExpandedMul).
		Int("nbExpandedAdd", lstats.NbExpandedAdd).
		Int("nbExpandedCst", lstats.NbExpandedCst).
		Int("nbVariables", lstats.NbTotGates).
		Int("nbUsedVariables", lstats.NbUsedGates).
		Msg("optimized layered circuit")
	res := CompileResult{
		rc:         rc,
		compiled:   lrc,
		inputOrder: io,
	}
	return &res, nil
}

// ProfilingCompile compiles the given circuit with profiling enabled, outputting the cost of each line of code.
// It does not return a compilation result as it does not complete the actual compilation process.
// Profiling is useful for performance analysis and optimization.
// TODO: Add support for sub-circuit profiling.
func ProfilingCompile(field *big.Int, circuit frontend.Circuit, opts ...frontend.CompileOption) error {
	var root *builder.ProfilingRoot
	newBuilder_ := func(field *big.Int, config frontend.CompileConfig) (frontend.Builder, error) {
		if root != nil {
			panic("newBuilder can only be called once")
		}
		root = builder.NewProfilingRoot(field, config)
		return root.GetRootBuilder(), nil
	}
	// returned R1CS is useless
	_, err := frontend.Compile(field, newBuilder_, circuit, opts...)
	// make sure gkr.API is implemented by ProfilingBuilder
	_ = API(root.GetRootBuilder())
	if err != nil {
		return err
	}
	rc, varSourceInfo := root.Finalize()
	if err := ir.ValidateForLayering(rc); err != nil {
		return err
	}
	varCost := layering.ProfilingCompile(rc)
	utils.ShowProfiling(varSourceInfo, varCost)
	return nil
}

// GetCircuitIr returns the intermediate representation (IR) of the compiled circuit as *ir.RootCircuit.
func (c *CompileResult) GetCircuitIr() *ir.RootCircuit {
	return c.rc
}

// GetInputSolver returns the InputSolver component of the compilation result as *ir.InputSolver.
func (c *CompileResult) GetLayeredCircuit() *layered.RootCircuit {
	return c.compiled
}

// GetLayeredCircuit returns the Layered Circuit component of the compilation result as *layered.RootCircuit.
func (c *CompileResult) GetInputSolver() *ir.InputSolver {
	return ir.GetInputSolver(c.rc, c.inputOrder)
}

// DeserializeLayeredCircuit takes a byte buffer and returns a pointer to a layered.RootCircuit
// which represents a deserialized layered circuit.
func DeserializeLayeredCircuit(buf []byte) *layered.RootCircuit {
	return layered.DeserializeRootCircuit(buf)
}

// DeserializeInputSolver takes a byte buffer and returns a pointer to an ir.InputSolver
// which represents a deserialized input solver.
func DeserializeInputSolver(buf []byte) *ir.InputSolver {
	return ir.DeserializeInputSolver(buf)
}
