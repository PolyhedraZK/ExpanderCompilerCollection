

/*

type ConvertValidatorListToMerkleTreeCircuit struct {
	ValidatorHashChunk [SUBTREESIZE][hash.PoseidonHashLength]frontend.Variable
	SubtreeRoot        [hash.PoseidonHashLength]frontend.Variable `gnark:",public"`
}

func (circuit *ConvertValidatorListToMerkleTreeCircuit) Define(api frontend.API) error {
	inputs := make([]frontend.Variable, 0)
	for i := 0; i < len(circuit.ValidatorHashChunk); i++ {
		inputs = append(inputs, circuit.ValidatorHashChunk[i][:]...)
	}
	subTreeRoot := hash.GenericHash(api, inputs, hash.PoseidonHashLength)
	for i := 0; i < hash.PoseidonHashLength; i++ {
		api.AssertIsEqual(subTreeRoot[i], circuit.SubtreeRoot[i])
	}
	return nil
}

type MerkleSubTreeWithLimitCircuit struct {
	SubtreeRoot        [SUBTREENUM][hash.PoseidonHashLength]frontend.Variable `gnark:",public"`
	TreeRootMixIn      [hash.PoseidonHashLength]frontend.Variable             `gnark:",public"`
	RealValidatorCount [8]frontend.Variable                                   `gnark:",public"` //little-endian encoding
	TreeRoot           [hash.PoseidonHashLength]frontend.Variable
	Path               [PADDINGDEPTH]frontend.Variable
	Aunts              [PADDINGDEPTH][hash.PoseidonHashLength]frontend.Variable
}

func (circuit *MerkleSubTreeWithLimitCircuit) Define(api frontend.API) error {
	inputs := make([]frontend.Variable, 0)
	for i := 0; i < len(circuit.SubtreeRoot); i++ {
		inputs = append(inputs, circuit.SubtreeRoot[i][:]...)
	}
	subTreeRootRoot := hash.GenericHash(api, inputs, hash.PoseidonHashLength)
	aunts := make([][]frontend.Variable, len(circuit.Aunts))
	for i := 0; i < len(circuit.Aunts); i++ {
		aunts[i] = make([]frontend.Variable, len(circuit.Aunts[i]))
		copy(aunts[i][:], circuit.Aunts[i][:])
	}
	//make sure the merkle tree root is correct
	merkle.VerifyMerkleTreePathVariable(api, circuit.TreeRoot[:], subTreeRootRoot, circuit.Path[:], aunts, 0)
	//calculate the treeRootMixIn, which is held by the verifier and changed every epoch
	treeRootMixIn := hash.GenericHash(api, append(circuit.TreeRoot[:], circuit.RealValidatorCount[:]...), hash.PoseidonHashLength)
	for i := 0; i < hash.PoseidonHashLength; i++ {
		api.AssertIsEqual(treeRootMixIn[i], circuit.TreeRootMixIn[i])
	}
	return nil
}

*/