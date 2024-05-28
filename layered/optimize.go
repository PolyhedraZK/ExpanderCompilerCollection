package layered

import (
	"math/big"

	"github.com/Zklib/gkr-compiler/utils"
)

func sortMulGates(mul []GateMul) []GateMul {
	s := make([]int, len(mul))
	for i := 0; i < len(mul); i++ {
		s[i] = i
	}
	utils.SortIntSeq(s, func(i, j int) bool {
		if mul[i].Out != mul[j].Out {
			return mul[i].Out < mul[j].Out
		}
		if mul[i].In0 != mul[j].In0 {
			return mul[i].In0 < mul[j].In0
		}
		if mul[i].In1 != mul[j].In1 {
			return mul[i].In1 < mul[j].In1
		}
		return mul[i].Coef.Cmp(mul[j].Coef) < 0
	})
	res := make([]GateMul, len(mul))
	for i, j := range s {
		res[i] = mul[j]
	}
	return res
}

func sortAddGates(add []GateAdd) []GateAdd {
	s := make([]int, len(add))
	for i := 0; i < len(add); i++ {
		s[i] = i
	}
	utils.SortIntSeq(s, func(i, j int) bool {
		if add[i].Out != add[j].Out {
			return add[i].Out < add[j].Out
		}
		if add[i].In != add[j].In {
			return add[i].In < add[j].In
		}
		return add[i].Coef.Cmp(add[j].Coef) < 0
	})
	res := make([]GateAdd, len(add))
	for i, j := range s {
		res[i] = add[j]
	}
	return res
}

func sortCstGates(cst []GateCst) []GateCst {
	s := make([]int, len(cst))
	for i := 0; i < len(cst); i++ {
		s[i] = i
	}
	utils.SortIntSeq(s, func(i, j int) bool {
		if cst[i].Out != cst[j].Out {
			return cst[i].Out < cst[j].Out
		}
		return cst[i].Coef.Cmp(cst[j].Coef) < 0
	})
	res := make([]GateCst, len(cst))
	for i, j := range s {
		res[i] = cst[j]
	}
	return res
}

func dedupMulGates(mul []GateMul, field *big.Int) []GateMul {
	if len(mul) == 0 {
		return mul
	}
	res := []GateMul{mul[0]}
	for i := 1; i < len(mul); i++ {
		if mul[i].In0 != mul[i-1].In0 || mul[i].In1 != mul[i-1].In1 || mul[i].Out != mul[i-1].Out || field.Cmp(mul[i].Coef) <= 0 {
			res = append(res, mul[i])
		} else {
			res[len(res)-1].Coef.Add(res[len(res)-1].Coef, mul[i].Coef)
		}
	}
	return res
}

func dedupAddGates(add []GateAdd, field *big.Int) []GateAdd {
	if len(add) == 0 {
		return add
	}
	res := []GateAdd{add[0]}
	for i := 1; i < len(add); i++ {
		if add[i].In != add[i-1].In || add[i].Out != add[i-1].Out || field.Cmp(add[i].Coef) <= 0 {
			res = append(res, add[i])
		} else {
			res[len(res)-1].Coef.Add(res[len(res)-1].Coef, add[i].Coef)
		}
	}
	return res
}

func dedupCstGates(cst []GateCst, field *big.Int) []GateCst {
	if len(cst) == 0 {
		return cst
	}
	res := []GateCst{cst[0]}
	for i := 1; i < len(cst); i++ {
		if cst[i].Out != cst[i-1].Out || field.Cmp(cst[i].Coef) <= 0 {
			res = append(res, cst[i])
		} else {
			res[len(res)-1].Coef.Add(res[len(res)-1].Coef, cst[i].Coef)
		}
	}
	return res
}

func expandCircuit(circuit *Circuit, prevCircuits []*Circuit, field *big.Int, expandRange map[int]bool) *Circuit {
	res := &Circuit{
		InputLen:    circuit.InputLen,
		OutputLen:   circuit.OutputLen,
		SubCircuits: []SubCircuit{},
		Mul:         circuit.Mul,
		Add:         circuit.Add,
		Cst:         circuit.Cst,
	}
	for _, sub := range circuit.SubCircuits {
		subc := prevCircuits[sub.Id]
		if !expandRange[int(sub.Id)] {
			res.SubCircuits = append(res.SubCircuits, sub)
			continue
		}
		for _, al := range sub.Allocations {
			for _, m := range subc.Mul {
				res.Mul = append(res.Mul, GateMul{
					In0:  m.In0 + al.InputOffset,
					In1:  m.In1 + al.InputOffset,
					Out:  m.Out + al.OutputOffset,
					Coef: m.Coef,
				})
			}
			for _, a := range subc.Add {
				res.Add = append(res.Add, GateAdd{
					In:   a.In + al.InputOffset,
					Out:  a.Out + al.OutputOffset,
					Coef: a.Coef,
				})
			}
			for _, c := range subc.Cst {
				res.Cst = append(res.Cst, GateCst{
					Out:  c.Out + al.OutputOffset,
					Coef: c.Coef,
				})
			}
			for _, subsub := range subc.SubCircuits {
				x := 0
				for x != len(res.SubCircuits) && res.SubCircuits[x].Id != subsub.Id {
					x++
				}
				if x == len(res.SubCircuits) {
					res.SubCircuits = append(res.SubCircuits, SubCircuit{Id: subsub.Id})
				}
				for _, al2 := range subsub.Allocations {
					res.SubCircuits[x].Allocations = append(res.SubCircuits[x].Allocations, Allocation{
						InputOffset:  al.InputOffset + al2.InputOffset,
						OutputOffset: al.OutputOffset + al2.OutputOffset,
					})
				}
			}
		}
	}
	res.Mul = dedupMulGates(sortMulGates(res.Mul), field)
	res.Add = dedupAddGates(sortAddGates(res.Add), field)
	res.Cst = dedupCstGates(sortCstGates(res.Cst), field)
	return res
}

// expand small circuits and circuits which only occurs once
// also removed unused circuits
func optimize1(rc *RootCircuit) (*RootCircuit, bool) {
	const ExpandUseCountLimit = 1
	const ExpandGateCountLimit = 4
	inLayers := make([]bool, len(rc.Circuits))
	usedCount := make([]int, len(rc.Circuits))
	expandRange := make(map[int]bool)
	for _, x := range rc.Layers {
		// for final circuits, we add a large number to prevent it from being expanded
		usedCount[x] += ExpandUseCountLimit + 1
		inLayers[x] = true
	}
	for i := len(rc.Circuits) - 1; i >= 0; i-- {
		if usedCount[i] > 0 {
			for _, x := range rc.Circuits[i].SubCircuits {
				usedCount[x.Id] += len(x.Allocations)
			}
		}
	}
	optimized := false
	for i, c := range rc.Circuits {
		if usedCount[i] == 0 {
			optimized = true
			continue
		}
		if inLayers[i] {
			continue
		}
		// here gateCount includes gates and subcircuit allocations
		gateCount := len(c.Mul) + len(c.Add) + len(c.Cst)
		for _, sub := range c.SubCircuits {
			gateCount += len(sub.Allocations)
		}
		if usedCount[i] <= ExpandUseCountLimit || gateCount <= ExpandGateCountLimit {
			expandRange[i] = true
			optimized = true
		}
	}
	if !optimized {
		return rc, false
	}
	expandedCircuits := make([]*Circuit, len(rc.Circuits))
	for i, c := range rc.Circuits {
		if usedCount[i] > 0 {
			expandedCircuits[i] = expandCircuit(c, expandedCircuits, rc.Field, expandRange)
		} else {
			expandedCircuits[i] = c
		}
	}
	newId := make([]uint64, len(rc.Circuits))
	for i := 0; i < len(rc.Circuits); i++ {
		newId[i] = ^uint64(0)
	}
	newCircuits := []*Circuit{}
	for i, c := range expandedCircuits {
		if usedCount[i] > 0 && !expandRange[i] {
			for j := range c.SubCircuits {
				c.SubCircuits[j].Id = newId[c.SubCircuits[j].Id]
			}
			newId[i] = uint64(len(newCircuits))
			newCircuits = append(newCircuits, c)
		}
	}
	newLayers := make([]uint64, len(rc.Layers))
	for i := range rc.Layers {
		newLayers[i] = newId[rc.Layers[i]]
	}
	return &RootCircuit{
		Field:    rc.Field,
		Circuits: newCircuits,
		Layers:   newLayers,
	}, true
}

// TODO: implement more optimization strategies: remove unused gates, dedupe exactly same circuits

// Optimize applies various optimization strategies to a RootCircuit to reduce its
// complexity and improve computational efficiency.
func Optimize(rc *RootCircuit) *RootCircuit {
	for {
		nrc, ok := optimize1(rc)
		if !ok {
			return nrc
		}
		rc = nrc
	}
}
