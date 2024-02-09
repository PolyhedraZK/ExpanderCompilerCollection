package gkr

import (
	"math/big"

	"github.com/Zklib/gkr-compiler/builder"
	"github.com/Zklib/gkr-compiler/ir"
	"github.com/Zklib/gkr-compiler/layered"
	"github.com/Zklib/gkr-compiler/layering"
	"github.com/consensys/gnark/frontend"
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

func Compile(field *big.Int, circuit frontend.Circuit, pad2n bool, opts ...frontend.CompileOption) (*CompileResult, error) {
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
	if err := ir.Validate(rc); err != nil {
		return nil, err
	}
	rc = ir.AdjustForLayering(rc)
	if err := ir.ValidateForLayering(rc); err != nil {
		return nil, err
	}
	lrc, is := layering.Compile(rc)
	if err := layered.Validate(lrc); err != nil {
		return nil, err
	}
	if err := layered.ValidateInitialized(lrc); err != nil {
		return nil, err
	}
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
