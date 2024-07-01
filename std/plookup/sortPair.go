package plookup

import (
	"math/big"
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
