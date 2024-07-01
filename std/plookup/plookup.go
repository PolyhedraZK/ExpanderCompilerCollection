package plookup

import (
	"fmt"
	"math/big"
	"sort"

	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
)

var (
	UsedVariables           [][]frontend.Variable
	MultiTableUsedVariables [][][]frontend.Variable
)

func init() {
	UsedVariables = make([][]frontend.Variable, 2)
	UsedVariables[0] = make([]frontend.Variable, 0)
	UsedVariables[1] = make([]frontend.Variable, 0)
	MultiTableUsedVariables = make([][][]frontend.Variable, 0)
	Hints := []solver.Hint{sort2DHint}
	solver.RegisterHint(Hints...)
}
func Reset() {
	UsedVariables = make([][]frontend.Variable, 2)
	UsedVariables[0] = make([]frontend.Variable, 0)
	UsedVariables[1] = make([]frontend.Variable, 0)
	MultiTableUsedVariables = make([][][]frontend.Variable, 0)
}

func sort2DHint(_ *big.Int, inputs []*big.Int, outputs []*big.Int) error {
	n := inputs[0].Int64()
	// tableSize := int(inputs[1].Int64())
	keys := make([]*big.Int, n/2)
	values := make([]*big.Int, n/2)
	// copy inputs to outputs
	for i := int64(0); i < n/2; i++ {
		keys[i] = new(big.Int).Set(inputs[i+2])
		values[i] = new(big.Int).Set(inputs[i+n/2+2])
	}
	valueIndexPairs := make(KeyValuePairs, n/2)
	for i := 0; i < int(n/2); i++ {
		valueIndexPairs[i] = KeyValuePair{Key: keys[i], Value: values[i]}
	}

	sort.Sort(valueIndexPairs)
	for i := 0; i < int(n/2); i++ {
		outputs[i] = valueIndexPairs[i].Key
		outputs[i+int(n/2)] = valueIndexPairs[i].Value
	}
	return nil
}

// this interface write to a single (default) table, UsedVariables
// row 0 (UsedVariables[0]) is the key
// row 1 (UsedVariables[1]) is the value
func NewTable(api frontend.API, key, value []frontend.Variable) error {
	if len(key) != len(value) {
		return fmt.Errorf("key and value should have the same length")
	}
	if len(UsedVariables) != 2 {
		return fmt.Errorf("UsedVariables should have length 2")
	}
	if len(UsedVariables[0]) != 0 || len(UsedVariables[1]) != 0 {
		return fmt.Errorf("UsedVariables[0] should be empty")
	}
	keyLen := frontend.Variable(len(key))
	UsedVariables[0] = append(UsedVariables[0], keyLen)
	UsedVariables[1] = append(UsedVariables[1], keyLen)
	for i := 0; i < len(key); i++ {
		UsedVariables[0] = append(UsedVariables[0], key[i])
		UsedVariables[1] = append(UsedVariables[1], value[i])
	}
	return nil
}

// this interface write a new table to a multitable, MultiTableUsedVariables
// the new table is appended to the end of the multitable
// row 0 (MultiTableUsedVariables[tableId][0]) is the key
// row 1 (MultiTableUsedVariables[tableId][1]) is the value
func NewMultiTable(api frontend.API, key, value []frontend.Variable) (int, error) {
	if len(key) != len(value) {
		return -1, fmt.Errorf("key and value should have the same table size")
	}
	if len(key) != len(value) {
		return -1, fmt.Errorf("key and value should have the same length")
	}
	currentTable := make([][]frontend.Variable, 2)
	currentTable[0] = make([]frontend.Variable, len(key)+1)
	currentTable[1] = make([]frontend.Variable, len(key)+1)

	keyLen := frontend.Variable(len(key))
	currentTable[0][0] = keyLen
	currentTable[1][0] = keyLen
	copy(currentTable[0][1:], key)
	copy(currentTable[1][1:], value)
	MultiTableUsedVariables = append(MultiTableUsedVariables, currentTable)
	return len(MultiTableUsedVariables) - 1, nil
}

// this interface append new table items (key, value) to the default table, UsedVariables
// row 0 (UsedVariables[0]) is the key
// row 1 (UsedVariables[1]) is the value
func Append(api frontend.API, key, value []frontend.Variable) error {
	if len(key) != len(value) {
		return fmt.Errorf("key and value should have the same length")
	}
	if len(UsedVariables) != 2 {
		return fmt.Errorf("UsedVariables should have length 2")
	}
	if len(UsedVariables[0]) == 0 || len(UsedVariables[1]) == 0 {
		UsedVariables[0] = append(UsedVariables[0], frontend.Variable(0))
		UsedVariables[1] = append(UsedVariables[1], frontend.Variable(0))
	}
	keyLen := frontend.Variable(len(key))
	UsedVariables[0][0] = api.Add(UsedVariables[0][0], keyLen)
	UsedVariables[1][0] = api.Add(UsedVariables[1][0], keyLen)
	for i := 0; i < len(key); i++ {
		UsedVariables[0] = append(UsedVariables[0], key[i])
		UsedVariables[1] = append(UsedVariables[1], value[i])
	}
	return nil
}

// this interface append new table items (key, value) to a specific table in the multitable, MultiTableUsedVariables
// row 0 (MultiTableUsedVariables[tableId][0]) is the key
// row 1 (MultiTableUsedVariables[tableId][1]) is the value
func AppendTargetTable(api frontend.API, tableId int, key, value []frontend.Variable) error {
	if len(key) != len(value) {
		return fmt.Errorf("key and value should have the same length")
	}
	if len(MultiTableUsedVariables[tableId]) != 2 {
		return fmt.Errorf("UsedVariables should have length 2")
	}
	if len(MultiTableUsedVariables[tableId][0]) == 0 || len(MultiTableUsedVariables[tableId][1]) == 0 {
		MultiTableUsedVariables[tableId][0] = append(MultiTableUsedVariables[tableId][0], frontend.Variable(0))
		MultiTableUsedVariables[tableId][1] = append(MultiTableUsedVariables[tableId][1], frontend.Variable(0))
	}
	keyLen := frontend.Variable(len(key))
	MultiTableUsedVariables[tableId][0][0] = api.Add(MultiTableUsedVariables[tableId][0][0], keyLen)
	MultiTableUsedVariables[tableId][1][0] = api.Add(MultiTableUsedVariables[tableId][1][0], keyLen)
	for i := 0; i < len(key); i++ {
		MultiTableUsedVariables[tableId][0] = append(MultiTableUsedVariables[tableId][0], key[i])
		MultiTableUsedVariables[tableId][1] = append(MultiTableUsedVariables[tableId][1], value[i])
	}
	return nil
}

// this interface write a query to the default table, UsedVariables
// a query is a pair of key and value
// key is appended to row 0 (UsedVariables[0])
// value is appended to row 1 (UsedVariables[1])
func Lookup(api frontend.API, key, value frontend.Variable) error {
	UsedVariables[0] = append(UsedVariables[0], key)
	UsedVariables[1] = append(UsedVariables[1], value)
	return nil
}

// this interface write a query to a specific table in the multitable, MultiTableUsedVariables
// a query is a pair of key and value
// key is appended to row 0 (MultiTableUsedVariables[tableId][0])
// value is appended to row 1 (MultiTableUsedVariables[tableId][1])
func LookupTargetTable(api frontend.API, tableId int, key, value frontend.Variable) error {
	if len(MultiTableUsedVariables) <= tableId {
		return fmt.Errorf("tableId %d is out of range", tableId)
	}
	MultiTableUsedVariables[tableId][0] = append(MultiTableUsedVariables[tableId][0], key)
	MultiTableUsedVariables[tableId][1] = append(MultiTableUsedVariables[tableId][1], value)
	return nil
}

// this interface write multiple query to the default table, UsedVariables
// key is appended to row 0 (UsedVariables[0])
// value is appended to row 1 (UsedVariables[1])
func LookupArray(api frontend.API, key, value []frontend.Variable) error {
	if len(key) != len(value) {
		return fmt.Errorf("key and value should have the same length")
	}
	UsedVariables[0] = append(UsedVariables[0], key...)
	UsedVariables[1] = append(UsedVariables[1], value...)
	return nil
}

// this interface write multiple query to a specific table in the multitable, MultiTableUsedVariables
// key is appended to row 0 (MultiTableUsedVariables[tableId][0])
// value is appended to row 1 (MultiTableUsedVariables[tableId][1])
func LookupArrayTargetTable(api frontend.API, tableId int, key, value []frontend.Variable) error {
	if len(key) != len(value) {
		return fmt.Errorf("key and value should have the same length")
	}
	if len(MultiTableUsedVariables) <= tableId {
		return fmt.Errorf("tableId %d is out of range", tableId)
	}
	MultiTableUsedVariables[tableId][0] = append(MultiTableUsedVariables[tableId][0], key...)
	MultiTableUsedVariables[tableId][1] = append(MultiTableUsedVariables[tableId][1], value...)
	return nil
}

// realize batchMul, reduce computation cost
func batchMul(api frontend.API, a []frontend.Variable) frontend.Variable {
	length := len(a)
	if length == 1 {
		return a[0]
	}
	for len(a) > 1 {
		newLen := (len(a) + 1) / 2
		newArr := make([]frontend.Variable, newLen)
		for i := 0; i < newLen; i++ {
			if i*2+1 == len(a) {
				newArr[i] = a[i*2]
			} else {
				newArr[i] = api.Mul(a[i*2], a[i*2+1])
			}
		}
		a = newArr
	}
	return a[0]
}

// compress “key-value“ array to value array, nBits is the number of bits to shift, it is the length of key
// “key-value“ array is a array with keys and values, the first half is keys, the second half is values (key1, key2, ..., keyN, value1, value2, ..., valueN)
func compressedKeyValueToValue(api frontend.API, compressed []frontend.Variable, nBits int) []frontend.Variable {
	length := len(compressed) / 2
	result := make([]frontend.Variable, length)
	for i := 0; i < len(compressed)/2; i++ {
		result[i] = api.Add(api.Mul(compressed[i+length], frontend.Variable(1<<nBits)), compressed[i])
	}
	return result
}
func FinalCheck(api frontend.API, r frontend.Variable) error {
	// sorting
	hintInput := make([]frontend.Variable, len(UsedVariables[0])+len(UsedVariables[1]))
	hintInput[0] = len(UsedVariables[0]) + len(UsedVariables[1]) - 2
	copy(hintInput[1:], UsedVariables[0])
	copy(hintInput[1+len(UsedVariables[0]):], UsedVariables[1][1:]) //omit the first element of UsedVariables[1]
	sortedVariables, err := api.Compiler().NewHint(sort2DHint, (len(UsedVariables[0])-1)*2, hintInput...)
	if err != nil {
		panic(err)
	}
	// api.Println("sortedVariables", sortedVariables)
	// Permutation check
	sortedVariables1D := compressedKeyValueToValue(api, sortedVariables, 20)
	UsedVariables2D := make([]frontend.Variable, (len(UsedVariables[0])-1)*2)
	copy(UsedVariables2D, UsedVariables[0][1:])
	copy(UsedVariables2D[len(UsedVariables[0])-1:], UsedVariables[1][1:])
	UsedVariables1D := compressedKeyValueToValue(api, UsedVariables2D, 20)
	sub1 := make([]frontend.Variable, len(UsedVariables1D))
	sub2 := make([]frontend.Variable, len(UsedVariables1D))
	for i := 0; i < len(UsedVariables1D); i++ {
		sub1[i] = api.Sub(r, UsedVariables1D[i])
		sub2[i] = api.Sub(r, sortedVariables1D[i])
	}
	sumSub := frontend.Variable(0)
	for i := 0; i < len(UsedVariables1D); i++ {
		sumSub = api.Add(sumSub, api.Sub(sub1[i], sub2[i]))
	}
	fmt.Println("milestone 1")

	s0 := batchMul(api, sub1[:])
	fmt.Println("milestone 2")
	s1 := batchMul(api, sub2[:])
	fmt.Println("milestone 3")
	api.AssertIsEqual(s0, s1)
	fmt.Println("milestone 4")
	//diff := frontend.Variable(0)
	for i := 1; i < len(sortedVariables)/2; i++ {
		checkFlag := api.Sub(sortedVariables[i], sortedVariables[i-1])
		api.AssertIsBoolean(checkFlag)
		valueDiff := api.Sub(sortedVariables[i+len(sortedVariables)/2], sortedVariables[i-1+len(sortedVariables)/2])
		//diff = api.Or(diff, api.Select(checkFlag, 0, valueDiff))
		api.AssertIsEqual(0, api.Select(checkFlag, 0, valueDiff))
	}
	fmt.Println("milestone 5")
	return nil
}

func FinalCheckMultiTable(api frontend.API, r frontend.Variable) error {
	// sorting
	for i := 0; i < len(MultiTableUsedVariables); i++ {
		hintInput := make([]frontend.Variable, len(MultiTableUsedVariables[i][0])+len(MultiTableUsedVariables[i][1]))
		hintInput[0] = len(MultiTableUsedVariables[i][0]) + len(MultiTableUsedVariables[i][1]) - 2
		copy(hintInput[1:], MultiTableUsedVariables[i][0])
		copy(hintInput[1+len(MultiTableUsedVariables[i][0]):], MultiTableUsedVariables[i][1][1:]) //omit the first element of UsedVariables[1]
		sortedVariables, err := api.Compiler().NewHint(sort2DHint, (len(MultiTableUsedVariables[i][0])-1)*2, hintInput...)
		if err != nil {
			panic(err)
		}
		//tag0 := api.Tag("tag0")
		// Permutation check
		// Randomness from the anemoi-hash
		fmt.Println("milestone1")
		//tag1 := api.Tag("tag1")
		//api.AddCounter(tag0, tag1)
		// Check that the permutation is correct
		sortedVariables1D := compressedKeyValueToValue(api, sortedVariables, 20)
		UsedVariables2D := make([]frontend.Variable, (len(MultiTableUsedVariables[i][0])-1)*2)
		copy(UsedVariables2D, MultiTableUsedVariables[i][0][1:])
		copy(UsedVariables2D[len(MultiTableUsedVariables[i][0])-1:], MultiTableUsedVariables[i][1][1:])
		UsedVariables1D := compressedKeyValueToValue(api, UsedVariables2D, 20)
		sub1 := make([]frontend.Variable, len(UsedVariables1D))
		sub2 := make([]frontend.Variable, len(UsedVariables1D))
		// sub3 := make([]frontend.Variable, len(UsedVariables1D))
		for i := 0; i < len(UsedVariables1D); i++ {
			sub1[i] = api.Sub(r, UsedVariables1D[i])
			sub2[i] = api.Sub(r, sortedVariables1D[i])
		}
		sumSub := frontend.Variable(0)
		for i := 0; i < len(UsedVariables1D); i++ {
			sumSub = api.Add(sumSub, api.Sub(sub1[i], sub2[i]))
		}
		fmt.Println("milestone 2")

		s0 := batchMul(api, sub1[:])
		fmt.Println("milestone 3")
		s1 := batchMul(api, sub2[:])
		api.AssertIsEqual(s0, s1)
		for i := 1; i < len(sortedVariables)/2; i++ {
			checkFlag := api.Sub(sortedVariables[i], sortedVariables[i-1])
			api.AssertIsBoolean(checkFlag)
			valueDiff := api.Sub(sortedVariables[i+len(sortedVariables)/2], sortedVariables[i-1+len(sortedVariables)/2])
			api.AssertIsEqual(0, api.Select(checkFlag, 0, valueDiff))
		}
	}
	return nil
}
