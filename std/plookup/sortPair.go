package plookup

import (
	"fmt"
	"math/big"
	"sort"
)

type KeyValuePair struct {
	Key   *big.Int
	Value *big.Int
}

type KeyValuePairs []KeyValuePair

func (vip KeyValuePairs) Len() int {
	return len(vip)
}

func (vip KeyValuePairs) Swap(i, j int) {
	vip[i], vip[j] = vip[j], vip[i]
}

func (vip KeyValuePairs) Less(i, j int) bool {
	return vip[i].Key.Cmp(vip[j].Key) < 0
}

func Testmain() {
	arr := []*big.Int{
		big.NewInt(64),
		big.NewInt(34),
		big.NewInt(25),
		big.NewInt(12),
		big.NewInt(22),
		big.NewInt(11),
		big.NewInt(90),
	}

	valueIndexPairs := make(KeyValuePairs, len(arr))
	for i, v := range arr {
		valueIndexPairs[i] = KeyValuePair{Key: v, Value: big.NewInt(int64(i))}
	}

	sort.Sort(valueIndexPairs)

	sortedIndexes := make([]*big.Int, len(arr))
	for i, pair := range valueIndexPairs {
		sortedIndexes[i] = pair.Value
	}

	fmt.Println("Sorted values and their original indexes:")
	for _, pair := range valueIndexPairs {
		fmt.Printf("Value: %s, Original Index: %s\n", pair.Key.String(), pair.Value.String())
	}

	fmt.Println("Sorted indexes:")
	for _, index := range sortedIndexes {
		fmt.Println(index.String())
	}
}
