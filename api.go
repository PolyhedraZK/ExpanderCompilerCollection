package gkr

import (
	"math/big"

	"github.com/Zklib/gkr-compiler/builder"
	"github.com/Zklib/gkr-compiler/ir"
	"github.com/Zklib/gkr-compiler/layered"
	"github.com/Zklib/gkr-compiler/layering"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/logger"
)

type API interface {
	frontend.API
	builder.SubCircuitAPI
}

type CompileResult struct {
	rc          *ir.RootCircuit
	compiled    *layered.RootCircuit
	inputSolver *ir.InputSolver
}

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
	rc = ir.Optimize(rc)
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
	lrc, is := layering.Compile(rc)
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
		rc:          rc,
		compiled:    lrc,
		inputSolver: is,
	}
	return &res, nil
}

func (c *CompileResult) GetCircuitIr() *ir.RootCircuit {
	return c.rc
}

func (c *CompileResult) GetLayeredCircuit() *layered.RootCircuit {
	return c.compiled
}

func (c *CompileResult) GetWitness(assignment frontend.Circuit) []*big.Int {
	return c.rc.SolveInput(assignment, c.inputSolver)
}
