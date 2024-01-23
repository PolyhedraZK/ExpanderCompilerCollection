package gkr

import (
	"fmt"
	"math/big"
	"strconv"
	"strings"

	"github.com/Zklib/gkr-compiler/gkr/expr"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/logger"
)

type compileResult struct {
	builder *builder
}

func Compile(field *big.Int, circuit frontend.Circuit, pad2n bool, opts ...frontend.CompileOption) (*compileResult, error) {
	var builder *builder
	newBuilder_ := func(field *big.Int, config frontend.CompileConfig) (frontend.Builder, error) {
		if builder != nil {
			panic("newBuilder can only be called once")
		}
		builder = newBuilder(field, config)
		return builder, nil
	}
	// returned R1CS is useless
	_, err := frontend.Compile(field, newBuilder_, circuit, opts...)
	if err != nil {
		return nil, err
	}
	log := logger.Logger()
	log.Info().
		Int("nbConstraints", len(builder.constraints)).
		Int("nbInternal", builder.cs.GetNbInternalVariables()).
		Int("nbInput", builder.nbInput).
		Msg("built basic circuit")
	builder.finalize()
	log.Info().
		Int("nbInternal", builder.cs.GetNbInternalVariables()).
		Int("nbInput", builder.nbInput).
		Int("estimatedLayer", builder.vLayer[builder.output]).
		Msg("constraints finalized")
	builder.compile(pad2n)
	stats := builder.circuit.getStats()
	log.Info().
		Int("nbHybridArg", stats.hybridArgCount).
		Int("nbHybrid", stats.hybridCount).
		Int("nbRelay", stats.relayCount).
		Int("nbInput", stats.inputCount).
		Int("layers", stats.layers).
		Msg("compiled")
	res := compileResult{
		builder: builder,
	}
	return &res, nil
}

func (c *compileResult) GetWitness(assignment frontend.Circuit) witness {
	return c.builder.getWitness(assignment)
}

func (c *compileResult) GetLayeredCircuit() circuit {
	return c.builder.circuit
}

func (c *compileResult) Print() {
	builder := c.builder

	varToStr := func(e expr.Expression) string {
		s := make([]string, len(e))
		for i, term := range e {
			coeff := builder.cs.ToBigInt(term.Coeff).String()
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

	for _, hint_ := range builder.hints {
		if hint_.f == nil {
			fmt.Printf("v%d = %s\n", hint_.outputIds[0], varToStr(hint_.inputs[0]))
		} else {
			strs := make([]string, len(hint_.inputs))
			for i, x := range hint_.inputs {
				strs[i] = varToStr(x)
			}
			fmt.Printf("v%d = %s(%s)\n", hint_.outputIds[0], solver.GetHintName(hint_.f), strings.Join(strs, ","))
		}
	}
}
