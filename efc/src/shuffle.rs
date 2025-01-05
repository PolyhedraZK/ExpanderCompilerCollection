

/*

type ShuffleWithHashMapAggPubkeyCircuit struct {
	StartIndex         frontend.Variable
	ChunkLength        frontend.Variable
	ShuffleIndices     [ValidatorChunkSize]frontend.Variable `gnark:",public"`
	CommitteeIndices   [ValidatorChunkSize]frontend.Variable `gnark:",public"`
	Pivots             [ShuffleRound]frontend.Variable
	IndexCount         frontend.Variable
	PositionResults    [ShuffleRound * ValidatorChunkSize]frontend.Variable           // the curIndex -> curPosition Table
	PositionBitResults [ShuffleRound * ValidatorChunkSize]frontend.Variable           `gnark:",public"` // mimic a hint: query the positionBitSortedResults table with runtime positions, get the flip bits
	FlipResults        [ShuffleRound * ValidatorChunkSize]frontend.Variable           // mimic a hint: get the flips, but we will ensure the correctness of the flipResults in the funciton (CheckPhasesAndResults)
	ValidatorHashes    [ValidatorChunkSize][hash.PoseidonHashLength]frontend.Variable `gnark:",public"`
	Slot               frontend.Variable                                              //the pre-pre beacon root
	AggregationBits    [ValidatorChunkSize]frontend.Variable                          //the aggregation bits
	AggregatedPubkey   sw_bls12381_m31.G1Affine                                       `gnark:",public"` //the aggregated pubkey of this committee, used for later signature verification circuit
	AttestationBalance [8]frontend.Variable                                           `gnark:",public"` //the attestation balance of this committee, the accBalance of each effective attestation should be supermajority, > 2/3 total balance
}

func (circuit *ShuffleWithHashMapAggPubkeyCircuit) Define(api frontend.API) error {
	logup.Reset()
	curValidatorExp := int(math.Ceil(math.Log2(float64(MaxValidator))))
	logup.NewRangeProof(curValidatorExp)

	indicesChunk := GetIndiceChunk(api, circuit.StartIndex, circuit.ChunkLength, ValidatorChunkSize)

	//set padding indices to 0
	for i := 0; i < len(indicesChunk); i++ {
		ignoreFlag := api.IsZero(api.Add(circuit.FlipResults[i], 1))
		indicesChunk[i] = api.Select(ignoreFlag, 0, indicesChunk[i])
	}
	//flip the indices based on the hashbit
	curIndices := make([]frontend.Variable, len(indicesChunk))
	copy(curIndices, indicesChunk[:])
	//flatten the loop to reduce the gkr layers
	copyCurIndices := common.CopyArray(api, curIndices)
	for i := 0; i < ShuffleRound; i++ {
		//flip the indices based on the hashbit
		curIndices = flipWithHashBits(api, circuit.Pivots[i], circuit.IndexCount, copyCurIndices, circuit.PositionResults[i*ValidatorChunkSize:(i+1)*ValidatorChunkSize], circuit.PositionBitResults[i*ValidatorChunkSize:(i+1)*ValidatorChunkSize], circuit.FlipResults[i*ValidatorChunkSize:(i+1)*ValidatorChunkSize])
		copyCurIndices = common.CopyArray(api, curIndices)
	}

	//check the final curIndices, should be equal to the shuffleIndex
	//cost: 3 * MaxValidator
	for i := 0; i < len(circuit.ShuffleIndices); i++ {
		isMinusOne := api.IsZero(api.Add(circuit.FlipResults[i], 1))
		curIndices[i] = api.Select(isMinusOne, circuit.ShuffleIndices[i], curIndices[i])
		// api.Println("ShuffleIndices", circuit.ShuffleIndices[i], curIndices[i])
		tmpRes := api.IsZero(api.Sub(circuit.ShuffleIndices[i], curIndices[i]))
		api.AssertIsEqual(tmpRes, 1)
	}

	//TODO: we need to use a lookup circuit to ensure that (shuffleIndices, committeeIndices) in the (shuffleindices, validatorIndices) table

	//at the same time, we use the circuit.CommitteeIndice (contain a committee's indices) to lookup the pubkey list
	pubkeyList, accBalance := LookupPubkeyListForCommitteeBySlot(api, circuit.CommitteeIndices[:], circuit.Slot, circuit.ValidatorHashes[:])
	//later, we may need to check the realBalance by using aggregationBits
	effectBalance := CalculateBalance(api, accBalance, circuit.AggregationBits[:])
	//make the effectBalance public
	for i := 0; i < len(effectBalance); i++ {
		api.AssertIsEqual(effectBalance[i], circuit.AttestationBalance[i])
	}

	pubkeyListBLS := make([]sw_bls12381_m31.G1Affine, len(pubkeyList))
	for i := 0; i < len(pubkeyList); i++ {
		pubkeyListBLS[i] = bls.ConvertToPublicKeyBLS(api, pubkeyList[i])
	}

	//aggregate the pubkey list
	attestation.AggregateAttestationPublicKey(api, pubkeyListBLS, circuit.AggregationBits[:], circuit.AggregatedPubkey)
	logup.FinalCheck(api, logup.ColumnCombineOption)
	// api.Println("Pass!")
	return nil
}
*/