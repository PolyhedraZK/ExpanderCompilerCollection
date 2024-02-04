package gkr

import (
	"fmt"
	"math/big"
	"strconv"
	"strings"

	"github.com/Zklib/gkr-compiler/builder"
	"github.com/Zklib/gkr-compiler/circuitir"
	"github.com/Zklib/gkr-compiler/expr"
	"github.com/Zklib/gkr-compiler/layered"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/logger"
)

type compileResult struct {
	rc      *circuitir.RootCircuit
	circuit *layered.Circuit
	idx     []int
}

func Compile(field *big.Int, circuit frontend.Circuit, pad2n bool, opts ...frontend.CompileOption) (*compileResult, error) {
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
	/*log.Info().
	Int("nbConstraints", len(builder.constraints)).
	Int("nbInternal", builder.cs.GetNbInternalVariables()).
	Int("nbInput", builder.nbInput).
	Msg("built basic circuit")*/
	rc := root.Finalize()
	/*log.Info().
	Int("nbInternal", builder.cs.GetNbInternalVariables()).
	Int("nbInput", builder.nbInput).
	Int("estimatedLayer", builder.vLayer[builder.output]).
	Msg("constraints finalized")*/
	c, idx := layered.Compile(rc, pad2n)
	stats := c.GetStats()
	log.Info().
		Int("nbHybridArg", stats.HybridArgCount).
		Int("nbHybrid", stats.HybridCount).
		Int("nbRelay", stats.RelayCount).
		Int("nbInput", stats.InputCount).
		Int("layers", stats.Layers).
		Msg("compiled")
	res := compileResult{
		rc:      rc,
		circuit: c,
		idx:     idx,
	}
	return &res, nil
}

func (c *compileResult) GetWitness(assignment frontend.Circuit) layered.Witness {
	values := c.rc.Eval(assignment)
	return layered.GetWitness(c.rc, c.idx, values, c.circuit)
}

func (c *compileResult) GetLayeredCircuit() *layered.Circuit {
	return c.circuit
}

func (c *compileResult) Print() {
	ci := c.rc.Circuits[0]

	varToStr := func(e expr.Expression) string {
		s := make([]string, len(e))
		for i, term := range e {
			coeff := c.rc.Field.ToBigInt(term.Coeff).String()
			if term.VID0 == 0 {
				s[i] = coeff
			} else if term.VID1 == 0 {
				s[i] = "v" + strconv.Itoa(term.VID0) + "*" + coeff
			} else {
				s[i] = "v" + strconv.Itoa(term.VID0) + "*v" + strconv.Itoa(term.VID1) + "*" + coeff
			}
		}
		return strings.Join(s, "+")
	}

	fmt.Printf("%d\n", len(ci.Instructions))
	for _, insn := range ci.Instructions {
		if insn.Type == circuitir.IInternalVariable {
			fmt.Printf("v%d = %s\n", insn.OutputIds[0], varToStr(insn.Inputs[0]))
		} else {
			strs := make([]string, len(insn.Inputs))
			for i, x := range insn.Inputs {
				strs[i] = varToStr(x)
			}
			fmt.Printf("v%d", insn.OutputIds[0])
			for i := 1; i < len(insn.OutputIds); i++ {
				fmt.Printf(",v%d", insn.OutputIds[i])
			}
			fmt.Printf(" = %s(%s)\n", solver.GetHintName(insn.HintFunc), strings.Join(strs, ","))
		}
	}
}
