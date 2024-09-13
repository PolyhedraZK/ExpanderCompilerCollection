package poseidon

import (
	"github.com/PolyhedraZK/ExpanderCompilerCollection/field/m31"
	"github.com/consensys/gnark/constraint"
)

// Implement poseidon2's m4 algorithm
// Notice that in Poseidon2 the MDS matrix is implicit and is not passed to the function
//
// This function is to validate correctness purpose only
// it is not optimized for performance.
func applyM4(engine m31.Field, state []constraint.Element) []constraint.Element {

	// let t0 = x[0] + x[1];
	t0 := engine.Add(state[0], state[1])

	// let t02 = t0 + t0;
	t02 := engine.Add(t0, t0)

	// let t1 = x[2] + x[3];
	t1 := engine.Add(state[2], state[3])

	// let t12 = t1 + t1;
	t12 := engine.Add(t1, t1)

	// let t2 = x[1] + x[1] + t1;
	t2 := engine.Add(state[1], state[1])
	t2 = engine.Add(t2, t1)

	// let t3 = x[3] + x[3] + t0;
	t3 := engine.Add(state[3], state[3])
	t3 = engine.Add(t3, t0)

	// let t4 = t12 + t12 + t3;
	t4 := engine.Add(t12, t12)
	t4 = engine.Add(t4, t3)

	// let t5 = t02 + t02 + t2;
	t5 := engine.Add(t02, t02)
	t5 = engine.Add(t5, t2)

	// let t6 = t3 + t5;
	t6 := engine.Add(t3, t5)

	// let t7 = t2 + t4;
	t7 := engine.Add(t2, t4)

	return []constraint.Element{t6, t7, t4, t5}
}

// Notice that in Poseidon2 the MDS matrix is implicit and is not passed to the function
//
// This function is to validate correctness purpose only
// it is not optimized for performance.
func applyPoseidon2ExternalRoundMatrix(engine m31.Field, state []constraint.Element) []constraint.Element {
	// // Applies circ(2M4, M4, M4, M4).
	// for i in 0..4 {
	//     [
	//         state[4 * i],
	//         state[4 * i + 1],
	//         state[4 * i + 2],
	//         state[4 * i + 3],
	//     ] = apply_m4([
	//         state[4 * i],
	//         state[4 * i + 1],
	//         state[4 * i + 2],
	//         state[4 * i + 3],
	//     ]);
	// }
	output_state := make([]constraint.Element, 16)
	for i := 0; i < 4; i++ {
		tmp := applyM4(engine, state[4*i:4*i+4])
		for j := 0; j < 4; j++ {
			output_state[4*i+j] = tmp[j]
		}
	}

	// for j in 0..4 {
	//     let s = state[j] + state[j + 4] + state[j + 8] + state[j + 12];
	//     for i in 0..4 {
	//         state[4 * i + j] += s;
	//     }
	// }
	for j := 0; j < 4; j++ {
		s := engine.Add(engine.Add(state[j], state[j+4]), engine.Add(state[j+8], state[j+12]))

		for i := 0; i < 4; i++ {
			output_state[4*i+j] = engine.Add(output_state[4*i+j], s)
		}
	}
	return output_state
}

// Notice that in Poseidon2 the MDS matrix is implicit and is not passed to the function
//
// This function is to validate correctness purpose only
// it is not optimized for performance.
func applyPoseidon2InternalRoundMatrix(engine m31.Field, state []constraint.Element) []constraint.Element {
	// let sum = state[1..].iter().fold(state[0], |acc, s| acc + *s);
	// state.iter_mut().enumerate().for_each(|(i, s)| {
	//     // TODO(spapini): Change to rotations.
	//     *s = *s * BaseField::from_u32_unchecked(1 << (i + 1)) + sum;
	// });
	output_state := make([]constraint.Element, 16)
	copy(output_state, state)
	sum := state[0]
	for i := 1; i < 16; i++ {
		sum = engine.Add(sum, state[i])
	}
	for i := 0; i < 16; i++ {
		state[i] = engine.Add(engine.Mul(state[i], constraint.Element{uint64(1 << (i + 1))}), sum)
	}
	return state
}

// Poseidon2 hash function over M31 field.
func Poseidon2M31(param *PoseidonParams, input []constraint.Element) constraint.Element {
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
