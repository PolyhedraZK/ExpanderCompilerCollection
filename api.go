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
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
)

type API interface {
	frontend.API
	builder.SubCircuitAPI
}

type compileResult struct {
	rc         *circuitir.RootCircuit
	compiled   *layered.RootCircuit
	inputSolve layered.InputSolve
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
	rc := root.Finalize()
	(&compileResult{rc: rc}).Print()
	r := layered.Compile(rc)
	res := compileResult{
		rc:         rc,
		compiled:   r.RootCircuit,
		inputSolve: r.InputSolve,
	}
	return &res, nil
}

func (c *compileResult) GetLayeredCircuit() *layered.RootCircuit {
	return c.compiled
}

func PrintCircuit(ci *circuitir.Circuit, field constraint.R1CS) {
	varToStr := func(e expr.Expression) string {
		s := make([]string, len(e))
		for i, term := range e {
			coeff := field.ToBigInt(term.Coeff).String()
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

	for _, insn := range ci.Instructions {
		if insn.Type == circuitir.IInternalVariable {
			fmt.Printf("v%d = %s\n", insn.OutputIds[0], varToStr(insn.Inputs[0]))
		} else if insn.Type == circuitir.IHint {
			strs := make([]string, len(insn.Inputs))
			for i, x := range insn.Inputs {
				strs[i] = varToStr(x)
			}
			fmt.Printf("v%d", insn.OutputIds[0])
			for i := 1; i < len(insn.OutputIds); i++ {
				fmt.Printf(",v%d", insn.OutputIds[i])
			}
			fmt.Printf(" = %s(%s)\n", solver.GetHintName(insn.HintFunc), strings.Join(strs, ","))
		} else if insn.Type == circuitir.ISubCircuit {
			strs := make([]string, len(insn.Inputs))
			for i, x := range insn.Inputs {
				strs[i] = varToStr(x)
			}
			fmt.Printf("v%d", insn.OutputIds[0])
			for i := 1; i < len(insn.OutputIds); i++ {
				fmt.Printf(",v%d", insn.OutputIds[i])
			}
			fmt.Printf(" = sub%d(%s)\n", insn.SubCircuitId, strings.Join(strs, ","))
		}
	}

	for i, e := range ci.Output {
		fmt.Printf("out%d = %s\n", i, varToStr(e))
	}
}

func (c *compileResult) Print() {
	for k, v := range c.rc.Circuits {
		fmt.Printf("Circuit %d nbIn=%d nbOut=%d =================\n", k, v.NbExternalInput, len(v.Output))
		PrintCircuit(v, c.rc.Field)
		fmt.Println()
	}
}
