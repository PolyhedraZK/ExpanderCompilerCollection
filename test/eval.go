package test

import (
	"crypto/rand"
	"math/big"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/layered"
)

// check if first output is zero
func CheckCircuit(rc *layered.RootCircuit, input []*big.Int) bool {
	out := EvalCircuit(rc, input)
	return out[0].Cmp(big.NewInt(0)) == 0
}

func EvalCircuit(rc *layered.RootCircuit, input []*big.Int) []*big.Int {
	if len(input) != int(rc.Circuits[rc.Layers[0]].InputLen) {
		panic("input length mismatch")
	}
	cur := input
	// for layer_i, id := range rc.Layers {
	for _, id := range rc.Layers {
		next := make([]*big.Int, rc.Circuits[id].OutputLen)
		for i := range next {
			next[i] = big.NewInt(0)
		}
		applyCircuit(rc, rc.Circuits[id], cur, next)
		cur = next
		for i := range cur {
			cur[i].Mod(cur[i], rc.Field)
		}
		// fmt.Printf("eval layer %d: %v\n", layer_i, cur[0])
	}
	return cur
}

func randInt() *big.Int {
	buf := make([]byte, 64) // just 2 times longer
	rand.Read(buf)
	return new(big.Int).SetBytes(buf)
}

func applyCircuit(rc *layered.RootCircuit, circuit *layered.Circuit, cur []*big.Int, next []*big.Int) {
	tmp := big.NewInt(0)
	for _, m := range circuit.Mul {
		coef := m.Coef
		if coef.Cmp(rc.Field) == 0 {
			coef = randInt()
		}
		tmp.Mul(cur[m.In0], cur[m.In1])
		next[m.Out].Add(next[m.Out], tmp.Mul(tmp, coef))
	}
	for _, a := range circuit.Add {
		coef := a.Coef
		if coef.Cmp(rc.Field) == 0 {
			coef = randInt()
		}
		next[a.Out].Add(next[a.Out], tmp.Mul(cur[a.In], coef))
	}
	for _, c := range circuit.Cst {
		coef := c.Coef
		if coef.Cmp(rc.Field) == 0 {
			coef = randInt()
		}
		next[c.Out].Add(next[c.Out], coef)
	}
	for _, sub := range circuit.SubCircuits {
		sc := rc.Circuits[sub.Id]
		for _, alloc := range sub.Allocations {
			applyCircuit(rc, sc,
				cur[alloc.InputOffset:alloc.InputOffset+sc.InputLen],
				next[alloc.OutputOffset:alloc.OutputOffset+sc.OutputLen],
			)
		}
	}
}
