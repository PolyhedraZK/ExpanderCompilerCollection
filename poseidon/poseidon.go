// Poseidon hash function, written in the layered circuit.
package poseidon

import (
	"github.com/PolyhedraZK/ExpanderCompilerCollection/field/m31"
	"github.com/consensys/gnark/constraint"
)

func sBox(engine m31.Field, f constraint.Element) constraint.Element {
	x2 := engine.Mul(f, f)
	x4 := engine.Mul(x2, x2)
	return engine.Mul(x4, f)
}

// Poseidon hash function over M31 field.
func PoseidonM31(param *PoseidonParams, input []constraint.Element) constraint.Element {
	// todo: pad the input if it is too short
	if len(input) != param.NumStates {
		panic("input length does not match the number of states in the Poseidon parameters")
	}

	state := input
	engine := m31.Field{}

	// Applies the full rounds.
	for i := 0; i < param.NumHalfFullRounds; i++ {
		for j := 0; j < param.NumStates; j++ {
			state[j] = engine.Add(state[j], engine.FromInterface(param.ExternalRoundConstant[j][i]))
		}
		// we use original poseidon mds method here
		// it seems to be more efficient than poseidon2 for us as it requires less number of additions
		state = applyMdsMatrix(engine, state, param.MdsMatrix)
		// applyExternalRoundMatrix(engine, state)
		for j := 0; j < param.NumStates; j++ {
			state[j] = sBox(engine, state[j])
		}
	}

	// Applies the first half of partial rounds.
	for i := 0; i < param.NumHalfPartialRounds; i++ {
		state[0] = engine.Add(state[0], engine.FromInterface(param.InternalRoundConstant[i]))
		// we use original poseidon mds method here
		// it seems to be more efficient than poseidon2 for us as it requires less number of additions
		state = applyMdsMatrix(engine, state, param.MdsMatrix)
		// applyInternalRoundMatrix(engine, state)
		state[0] = sBox(engine, state[0])
	}

	// Applies the second half of partial rounds.
	for i := 0; i < param.NumHalfPartialRounds; i++ {
		state[0] = engine.Add(state[0], engine.FromInterface(param.InternalRoundConstant[i+param.NumHalfPartialRounds]))
		// we use original poseidon mds method here
		// it seems to be more efficient than poseidon2 for us as it requires less number of additions
		state = applyMdsMatrix(engine, state, param.MdsMatrix)
		// applyInternalRoundMatrix(engine, state)
		state[0] = sBox(engine, state[0])
	}

	// Applies the full rounds.
	for i := 0; i < param.NumHalfFullRounds; i++ {
		for j := 0; j < param.NumStates; j++ {
			state[j] = engine.Add(state[j], engine.FromInterface(param.ExternalRoundConstant[j][i+param.NumHalfFullRounds]))
		}
		// we use original poseidon mds method here
		// it seems to be more efficient than poseidon2 for us as it requires less number of additions
		state = applyMdsMatrix(engine, state, param.MdsMatrix)
		// applyExternalRoundMatrix(engine, state)
		for j := 0; j < param.NumStates; j++ {
			state[j] = sBox(engine, state[j])
		}
	}

	return state[0]
}

// we use original poseidon mds method here
// it seems to be more efficient than poseidon2 for us as it requires less number of additions
func applyMdsMatrix(engine m31.Field, state []constraint.Element, mds [][]uint32) []constraint.Element {
	tmp := make([]constraint.Element, len(state))
	for i := 0; i < len(state); i++ {
		tmp[i] = engine.Mul(state[0], constraint.Element{uint64(mds[i][0])})
		for j := 1; j < len(state); j++ {
			tmp[i] = engine.Add(tmp[i], engine.Mul(state[j], constraint.Element{uint64(mds[i][j])}))
		}
	}
	return tmp
}
