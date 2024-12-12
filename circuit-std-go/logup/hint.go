package logup

import (
	"math/big"
)

func rangeProofHint(q *big.Int, inputs []*big.Int, outputs []*big.Int) error {
	n := inputs[0].Int64()
	a := new(big.Int).Set(inputs[1])

	for i := int64(0); i < n/int64(LookupTableBits); i++ {
		a, outputs[i] = new(big.Int).DivMod(a, big.NewInt(int64(1<<LookupTableBits)), new(big.Int))
	}
	return nil
}

// TODO: Do we need bits check for the count?
func QueryCountHintFn(field *big.Int, inputs []*big.Int, outputs []*big.Int) error {
	for i := 0; i < len(outputs); i++ {
		outputs[i] = big.NewInt(0)
	}

	for i := 0; i < len(inputs); i++ {
		query_id := inputs[i].Int64()
		outputs[query_id].Add(outputs[query_id], big.NewInt(1))
	}
	return nil
}

func QueryCountBaseKeysHintFn(field *big.Int, inputs []*big.Int, outputs []*big.Int) error {
	for i := 0; i < len(outputs); i++ {
		outputs[i] = big.NewInt(0)
	}
	tableSize := inputs[0].Int64()
	table := inputs[1 : tableSize+1]
	queryKeys := inputs[tableSize+1:]
	for i := 0; i < len(queryKeys); i++ {
		queryKey := queryKeys[i].Int64()
		//find the location of the query key in the table
		for j := 0; j < len(table); j++ {
			if table[j].Int64() == queryKey {
				outputs[j].Add(outputs[j], big.NewInt(1))
			}
		}
	}
	return nil
}
