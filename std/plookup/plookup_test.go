package plookup

import (
	"fmt"
	"math/big"
	"testing"
	"time"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
)

const (
	NumberTable      = 3
	Round            = 3
	LookupTableSize  = 1024 / 256 * 256
	LookupTableSize2 = 1024 / 256 * 256
	LookupSize       = LookupTableSize * 256
	LookupSize2      = LookupTableSize2 * 256
)

type LookupCircuit struct {
	Key         [LookupTableSize]frontend.Variable
	Value       [LookupTableSize]frontend.Variable
	R           frontend.Variable
	LookupKey   [LookupSize]frontend.Variable
	LookupValue [LookupSize]frontend.Variable
	//HashOutputs  [HashMapSize][OutputLength]frontend.Variable `gnark:",public"`
}

func (circuit *LookupCircuit) Define(api frontend.API) error {
	Reset()
	NewTable(api, circuit.Key[:], circuit.Value[:])
	for i := 0; i < LookupSize; i++ {
		for j := 0; j < Round; j++ {
			Lookup(api, circuit.LookupKey[i], circuit.LookupValue[i])
		}
	}
	// randomness := api.(gkr.API).GetRandomValue()
	randomness := circuit.R
	FinalCheck(api, randomness)
	return nil
}

type LookupMultiTableCircuit struct {
	Keys             [NumberTable][LookupTableSize]frontend.Variable
	Values           [NumberTable][LookupTableSize]frontend.Variable
	KeySize2         [LookupTableSize]frontend.Variable
	ValueSize2       [LookupTableSize]frontend.Variable
	R                frontend.Variable
	LookupKeys       [NumberTable][LookupSize2]frontend.Variable
	LookupValues     [NumberTable][LookupSize2]frontend.Variable
	LookupKeySize2   [LookupSize2]frontend.Variable
	LookupValueSize2 [LookupSize2]frontend.Variable
	//HashOutputs  [HashMapSize][OutputLength]frontend.Variable `gnark:",public"`
}

func (circuit *LookupMultiTableCircuit) Define(api frontend.API) error {
	Reset()
	for i := 0; i < NumberTable; i++ {
		NewMultiTable(api, circuit.Keys[i][:], circuit.Values[i][:])
	}
	// PlookupWriteNewTable(api, circuit.Keys[NumberTable-1][:], circuit.Values[NumberTable-1][:])
	NewMultiTable(api, circuit.KeySize2[:], circuit.ValueSize2[:])
	for i := 0; i < NumberTable; i++ {
		LookupArrayTargetTable(api, i, circuit.LookupKeys[i][:], circuit.LookupValues[i][:])
	}
	LookupArrayTargetTable(api, NumberTable, circuit.LookupKeySize2[:], circuit.LookupValueSize2[:])
	// randomness := api.(gkr.API).GetRandomValue()
	randomness := circuit.R
	FinalCheckMultiTable(api, randomness)
	return nil
}
func gnarkProof(circuit, assignment frontend.Circuit) error {
	r1cs, err := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, circuit)
	if err != nil {
		fmt.Printf("Compile failed: %v\n", err)
		return err
	}
	pk, err := groth16.DummySetup(r1cs)
	if err != nil {
		fmt.Printf("Setup failed: %v\n", err)
		return err
	}
	validWitness, err := frontend.NewWitness(assignment, ecc.BN254.ScalarField())
	if err != nil {
		fmt.Printf("NewWitness failed: %v\n", err)
		return err
	}
	fmt.Println("Prove ...")
	startTime := time.Now()
	_, err = groth16.Prove(r1cs, pk, validWitness)
	endTime := time.Now()
	fmt.Println("Prove time:", endTime.Sub(startTime))
	if err != nil {
		fmt.Printf("Prove failed: %v\n", err)
		panic(err)
	}
	fmt.Println("Prove successful")
	return nil
}
func TestLookupCircuit(t *testing.T) {
	// frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &LookupCircuit{})
	var key [LookupTableSize]frontend.Variable
	var value [LookupTableSize]frontend.Variable
	var lookupKey [LookupSize]frontend.Variable
	var lookupValue [LookupSize]frontend.Variable
	offset := 1267
	for i := 0; i < LookupTableSize; i++ {
		key[i] = frontend.Variable(i + offset)
		value[i] = frontend.Variable(i * 1256)
	}
	//randon := 132441223
	for i := 0; i < LookupSize; i++ {
		index := (i) % LookupTableSize
		lookupKey[i] = key[index]
		lookupValue[i] = value[index]
	}
	assignment := new(LookupCircuit)
	assignment.Key = key
	assignment.Value = value
	assignment.LookupKey = lookupKey
	assignment.LookupValue = lookupValue
	assignment.R = frontend.Variable(LookupSize)
	gnarkProof(new(LookupCircuit), assignment)
	//have errors in gnark0.9.1
	// err := test.IsSolved(&LookupCircuit{}, &assignment, ecc.BN254.ScalarField())
	// if err != nil {
	// 	t.Fatal(err)
	// }
}
func TestWrongLookupCircuit(t *testing.T) {
	frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &LookupCircuit{})
	var key [LookupTableSize]frontend.Variable
	var value [LookupTableSize]frontend.Variable
	var lookupKey [LookupSize]frontend.Variable
	var lookupValue [LookupSize]frontend.Variable
	for i := 0; i < LookupTableSize; i++ {
		key[i] = frontend.Variable(i)
		value[i] = frontend.Variable(i * 1256)
	}
	//randon := 132441223
	for i := 0; i < LookupSize; i++ {
		index := (i) % LookupTableSize
		index2 := (i + 1) % LookupTableSize
		lookupKey[i] = key[index]
		lookupValue[i] = value[index2]
	}
	assignment := new(LookupCircuit)
	assignment.Key = key
	assignment.Value = value
	assignment.LookupKey = lookupKey
	assignment.LookupValue = lookupValue
	assignment.R = frontend.Variable(LookupSize)
	gnarkProof(new(LookupCircuit), assignment)
	//have errors in gnark0.9.1
	// err := test.IsSolved(&LookupCircuit{}, &assignment, ecc.BN254.ScalarField())
	// if err != nil {
	// 	t.Fatal(err)
	// }
}
func TestLookupMultiTableCircuit(t *testing.T) {
	// frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &LookupCircuit{})
	var key [LookupTableSize]frontend.Variable
	var value [LookupTableSize]frontend.Variable
	var lookupKey [LookupSize]frontend.Variable
	var lookupValue [LookupSize]frontend.Variable
	assignment := new(LookupMultiTableCircuit)
	//random := 132441223
	for i := 0; i < NumberTable; i++ {
		random := (i + 1) * 132441223
		for j := 0; j < LookupTableSize; j++ {
			key[j] = frontend.Variable(j)
			value[j] = frontend.Variable(j * random)
		}
		for j := 0; j < LookupSize; j++ {
			index := (j) % LookupTableSize
			lookupKey[j] = key[index]
			lookupValue[j] = value[index]
		}
		assignment.Keys[i] = key
		assignment.Values[i] = value
		assignment.LookupKeys[i] = lookupKey
		assignment.LookupValues[i] = lookupValue
	}

	var key2 [LookupTableSize2]frontend.Variable
	var value2 [LookupTableSize2]frontend.Variable
	var lookupKey2 [LookupSize2]frontend.Variable
	var lookupValue2 [LookupSize2]frontend.Variable
	random := 132441223
	for i := 0; i < LookupTableSize2; i++ {
		key2[i] = frontend.Variable(i)
		value2[i] = frontend.Variable(i * random)
	}
	for i := 0; i < LookupSize2; i++ {
		index := (i) % LookupTableSize2
		lookupKey2[i] = key2[index]
		lookupValue2[i] = value2[index]
	}
	assignment.KeySize2 = key2
	assignment.ValueSize2 = value2
	assignment.LookupKeySize2 = lookupKey2
	assignment.LookupValueSize2 = lookupValue2
	assignment.R = frontend.Variable(LookupSize)

	gnarkProof(new(LookupMultiTableCircuit), assignment)
	//have errors in gnark0.9.1
	// err := test.IsSolved(&LookupMultiTableCircuit{}, &assignment, ecc.BN254.ScalarField())
	// if err != nil {
	// 	t.Fatal(err)
	// }
}
func test2DSortHint(t *testing.T) {
	n := 5
	tableSize := 10
	inputs := make([]*big.Int, tableSize+n+2)
	for i := 0; i < tableSize; i++ {
		inputs[i+2] = big.NewInt(int64(i))
	}
	for i := 0; i < n; i++ {
		inputs[i+tableSize+2] = big.NewInt(int64(n - i))
	}
	for i := 0; i < tableSize; i++ {
		inputs = append(inputs, big.NewInt(int64(i)))
	}
	for i := 0; i < n; i++ {
		inputs = append(inputs, big.NewInt(int64(n-i)))
	}
	inputs[0] = big.NewInt(int64(n+tableSize) * 2)
	inputs[1] = big.NewInt(int64(tableSize))
	outputs := make([]*big.Int, (n+tableSize)*2)
	sort2DHint(big.NewInt(int64(n)), inputs, outputs)
	count := 0
	for i := 0; i < n+tableSize-1; i++ {
		//diff is outputs[i+1] - outputs[i]
		diff := new(big.Int).Sub(outputs[i+1], outputs[i])
		if diff.Cmp(big.NewInt(0)) == 0 {
			continue
		} else if diff.Cmp(big.NewInt(0)) > 0 {
			count++
		} else {
			t.Errorf("outputs[%d] = %d, outputs[%d] = %d, diff = %d", i+2, outputs[i+2], i+3, outputs[i+3], diff)
		}
	}
	if count != tableSize-1 {
		t.Errorf("count = %d, n = %d", count, n)
	}
}

// func TestSortHint(t *testing.T) {
// 	n := 5
// 	tableSize := 10
// 	inputs := make([]*big.Int, tableSize+n+2)
// 	outputs := make([]*big.Int, tableSize+n+2)
// 	for i := 0; i < tableSize; i++ {
// 		inputs[i+2] = big.NewInt(int64(i))
// 	}
// 	for i := 0; i < n; i++ {
// 		inputs[i+tableSize+2] = big.NewInt(int64(n - i))
// 	}
// 	inputs[0] = big.NewInt(int64(n + tableSize))
// 	inputs[1] = big.NewInt(int64(tableSize))
// 	sortHint(big.NewInt(int64(n)), inputs, outputs)
// 	count := 0
// 	for i := 0; i < n+tableSize-1; i++ {
// 		//diff is outputs[i+1] - outputs[i]
// 		diff := new(big.Int).Sub(outputs[i+1], outputs[i])
// 		if diff.Cmp(big.NewInt(0)) == 0 {
// 			continue
// 		} else if diff.Cmp(big.NewInt(0)) > 0 {
// 			count++
// 		} else {
// 			t.Errorf("outputs[%d] = %d, outputs[%d] = %d, diff = %d", i+2, outputs[i+2], i+3, outputs[i+3], diff)
// 		}
// 	}
// 	if count != tableSize-1 {
// 		t.Errorf("count = %d, n = %d", count, n)
// 	}
// }

// func TestLongSortHint(t *testing.T) {
// 	n := 5
// 	tableSize := 10
// 	inputs := make([]*big.Int, tableSize+n+2)
// 	outputs := make([]*big.Int, tableSize+n+2)
// 	var err bool
// 	for i := 0; i < tableSize; i++ {
// 		inputs[i+2], err = new(big.Int).SetString("1234567890123456789012345678900"+fmt.Sprint(i)+"0000000"+fmt.Sprint(i), 16)
// 		if !err {
// 			t.Errorf("error parsing hex string")
// 		}
// 	}
// 	for i := 0; i < n; i++ {
// 		inputs[i+tableSize+2], err = new(big.Int).SetString("1234567890123456789012345678900"+fmt.Sprint(n-i)+"0000000"+fmt.Sprint(n-i), 16)
// 		if !err {
// 			t.Errorf("error parsing hex string")
// 		}
// 	}
// 	inputs[0] = big.NewInt(int64(n + tableSize))
// 	inputs[1] = big.NewInt(int64(tableSize))
// 	sortHint(big.NewInt(int64(n)), inputs, outputs)
// 	count := 0
// 	for i := 0; i < n+tableSize-1; i++ {
// 		//diff is outputs[i+1] - outputs[i]
// 		diff := new(big.Int).Sub(outputs[i+1], outputs[i])
// 		if diff.Cmp(big.NewInt(0)) == 0 {
// 			continue
// 		} else if diff.Cmp(big.NewInt(0)) > 0 {
// 			count++
// 		} else {
// 			t.Errorf("outputs[%d] = %d, outputs[%d] = %d, diff = %d", i+2, outputs[i+2], i+3, outputs[i+3], diff)
// 		}
// 	}
// 	if count != tableSize-1 {
// 		t.Errorf("count = %d, n = %d", count, n)
// 	}
// }
