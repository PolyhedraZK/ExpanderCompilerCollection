package poseidon

import (
	"log"

	"github.com/PolyhedraZK/ExpanderCompilerCollection"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/field/m31"
	"github.com/consensys/gnark/frontend"
)

type PoseidonInternalStateVar struct {
	AfterHalfFullRound    [16]frontend.Variable
	AfterHalfPartialRound [16]frontend.Variable
	AfterPartialRound     [16]frontend.Variable
}

// Main function of proving poseidon in circuit.
//
// To obtain a more efficient layered circuit representation, we also feed the internal state of the hash to this function.
func PoseidonCircuit(
	api frontend.API,
	engine m31.Field,
	param *PoseidonParams,
	input []frontend.Variable,
	internalStateVars PoseidonInternalStateVar,
	useRandomness bool) frontend.Variable {
	// todo: pad the input if it is too short
	if len(input) != param.NumStates {
		log.Println("input length", len(input), "does not match the number of states in the Poseidon parameters")
		panic("")
	}

	// ============================
	// RLC of inputs
	// ============================
	r := [16]frontend.Variable{}
	if useRandomness {
		r[0] = frontend.Variable(1)
		r[1] = api.(ExpanderCompilerCollection.API).GetRandomValue()
		for i := 2; i < 16; i++ {
			r[i] = api.Mul(r[i-1], r[1])
		}
	}

	// ============================
	// Applies the full rounds.
	// ============================
	state := input
	for i := 0; i < param.NumHalfFullRounds; i++ {
		// add round constant
		for j := 0; j < param.NumStates; j++ {
			state[j] = api.Add(state[j], param.ExternalRoundConstant[j][i])
		}
		// apply affine transform
		tmp := applyMdsMatrixCircuit(api, state, param.MdsMatrix)
		// tmp := applyExternalRoundMatrix(api, state)
		state = tmp[:]
		// sbox
		for j := 0; j < param.NumStates; j++ {
			state[j] = sBoxCircuit(api, state[j])
		}
	}
	if useRandomness {
		rlc1 := innerProduct(api, r[:], internalStateVars.AfterHalfFullRound[:])
		rlc1_rec := innerProduct(api, r[:], state)
		api.AssertIsEqual(rlc1, rlc1_rec)
	} else {
		for i := 0; i < param.NumStates; i++ {
			api.AssertIsEqual(state[i], internalStateVars.AfterHalfFullRound[i])
		}
	}

	// ============================
	// Applies the first half of partial rounds.
	// ============================
	state = internalStateVars.AfterHalfFullRound[:]

	for i := 0; i < param.NumHalfPartialRounds; i++ {
		// add round constant
		state[0] = api.Add(state[0], param.InternalRoundConstant[i])
		// apply affine transform
		tmp := applyMdsMatrixCircuit(api, state, param.MdsMatrix)
		// tmp := applyInternalRoundMatrix(api, state)
		state = tmp[:]
		// sbox
		state[0] = sBoxCircuit(api, state[0])
	}

	if useRandomness {
		rlc2 := innerProduct(api, r[:], internalStateVars.AfterHalfPartialRound[:])
		rlc2_rec := innerProduct(api, r[:], state)
		api.AssertIsEqual(rlc2, rlc2_rec)
	} else {
		for i := 0; i < param.NumStates; i++ {
			api.AssertIsEqual(state[i], internalStateVars.AfterHalfPartialRound[i])
		}
	}

	// ============================
	// Applies the second half of partial rounds.
	// ============================
	state = internalStateVars.AfterHalfPartialRound[:]
	for i := 0; i < param.NumHalfPartialRounds; i++ {
		// add round constant
		state[0] = api.Add(state[0], param.InternalRoundConstant[i+param.NumHalfPartialRounds])
		// apply affine transform
		tmp := applyMdsMatrixCircuit(api, state, param.MdsMatrix)
		// tmp := applyInternalRoundMatrix(api, state)
		state = tmp[:]
		// sbox
		state[0] = sBoxCircuit(api, state[0])
	}

	if useRandomness {
		rlc3 := innerProduct(api, r[:], internalStateVars.AfterPartialRound[:])
		rlc3_rec := innerProduct(api, r[:], state)
		api.AssertIsEqual(rlc3, rlc3_rec)

	} else {
		for i := 0; i < param.NumStates; i++ {
			api.AssertIsEqual(state[i], internalStateVars.AfterPartialRound[i])
		}
	}

	// ============================
	// Applies the full rounds.
	// ============================
	state = internalStateVars.AfterPartialRound[:]
	for i := 0; i < param.NumHalfFullRounds; i++ {
		// add round constant
		for j := 0; j < param.NumStates; j++ {
			state[j] = api.Add(state[j], param.ExternalRoundConstant[j][i+param.NumHalfFullRounds])
		}
		// apply affine transform
		tmp := applyMdsMatrixCircuit(api, state, param.MdsMatrix)
		// tmp := applyExternalRoundMatrix(api, state)
		state = tmp[:]
		// sbox
		for j := 0; j < param.NumStates; j++ {
			state[j] = sBoxCircuit(api, state[j])
		}
	}

	return state[0]
}

func accumulate(api frontend.API, a []frontend.Variable) frontend.Variable {
	a1 := api.Add(a[0], a[1], a[2], a[3])
	a2 := api.Add(a[4], a[5], a[6], a[7])
	a3 := api.Add(a[8], a[9], a[10], a[11])
	a4 := api.Add(a[12], a[13], a[14], a[15])

	return api.Add(a1, a2, a3, a4)
}

func innerProduct(api frontend.API, a []frontend.Variable, b []frontend.Variable) frontend.Variable {
	var tmp [16]frontend.Variable
	for i := 0; i < len(a); i++ {
		tmp[i] = api.Mul(a[i], b[i])
	}
	return accumulate(api, tmp[:])
}

func applyMdsMatrixCircuit(api frontend.API, x []frontend.Variable, mds [][]uint32) [16]frontend.Variable {
	var res [16]frontend.Variable
	for i := 0; i < 16; i++ {
		var tmp [16]frontend.Variable
		for j := 0; j < 16; j++ {
			tmp[j] = api.Mul(x[j], mds[j][i])
		}
		res[i] = accumulate(api, tmp[:])
	}
	return res
}

// S-Box: raise element to the power of 5
func sBoxCircuit(api frontend.API, input frontend.Variable) frontend.Variable {
	t2 := api.Mul(input, input)
	t4 := api.Mul(t2, t2)
	return api.Mul(t4, input)
}
