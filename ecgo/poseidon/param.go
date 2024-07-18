package poseidon

type PoseidonParams struct {
	// number of full rounds
	NumFullRounds int
	// number of half full rounds
	NumHalfFullRounds int
	// number of partial rounds
	NumPartRounds int
	// number of half full rounds
	NumHalfPartialRounds int
	// number of states
	NumStates int
	// mds matrix
	MdsMatrix [][]uint32
	// external round constants
	ExternalRoundConstant [][]uint32
	// internal round constants
	InternalRoundConstant []uint32
}

// TODOs: the parameters are not secure. use a better way to generate the constants
func NewPoseidonParams() *PoseidonParams {
	num_full_rounds := 8
	num_part_rounds := 14
	num_states := 16

	external_round_constant := make([][]uint32, num_states)

	for i := 0; i < num_states; i++ {
		external_round_constant[i] = make([]uint32, num_full_rounds)

		for j := 0; j < num_full_rounds; j++ {
			external_round_constant[i][j] = 1234
		}
	}

	internal_round_constant := make([]uint32, num_part_rounds)
	for i := 0; i < num_part_rounds; i++ {
		internal_round_constant[i] = 1234
	}

	mds := make([][]uint32, num_states)
	for i := 0; i < num_states; i++ {
		mds[i] = make([]uint32, num_states)
		for j := 0; j < num_states; j++ {
			mds[i][j] = 1234
		}
	}

	return &PoseidonParams{
		NumFullRounds:         num_full_rounds,
		NumHalfFullRounds:     num_full_rounds / 2,
		NumPartRounds:         num_part_rounds,
		NumHalfPartialRounds:  num_part_rounds / 2,
		NumStates:             num_states,
		MdsMatrix:             mds,
		ExternalRoundConstant: external_round_constant,
		InternalRoundConstant: internal_round_constant,
	}
}
