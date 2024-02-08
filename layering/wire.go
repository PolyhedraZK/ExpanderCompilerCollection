package layering

import (
	"math/big"

	"github.com/Zklib/gkr-compiler/layered"
)

type layoutQuery struct {
	l      *layerLayout
	ctx    *compileContext
	varPos map[int]int
}

// get the sub circuit layout by filtering the variables
func (lq *layoutQuery) query(vs []int, f func(int) int, cid uint64, lid int) *subLayout {
	ps := make([]int, len(vs))
	l := 1 << 62
	r := -l
	for i, x := range vs {
		ps[i] = lq.varPos[x]
		if ps[i] < l {
			l = ps[i]
		}
		if ps[i] > r {
			r = ps[i]
		}
	}
	//fmt.Printf("@ %v %v %v %d\n", vs, lq.varPos, ps, lid)
	xor := l ^ r
	for xor&-xor != xor {
		xor &= xor - 1
	}
	var n int
	if xor == 0 {
		n = 1
	} else {
		n = xor << 1
	}
	offset := l & (^(n - 1))
	placement := make([]int, n)
	for i := 0; i < n; i++ {
		placement[i] = -1
	}
	for i := range vs {
		placement[ps[i]-offset] = f(i)
	}
	subl := &layerLayout{
		circuitId:      cid,
		layer:          lid,
		sparse:         false,
		size:           n,
		placementDense: placement,
	}
	if lid >= 0 {
		subl.SubsMap(lq.ctx.circuits[cid].lcs[lid].varMap)
	} else {
		subl.SubsMap(lq.ctx.circuits[cid].lcHint.varMap)
	}
	id := lq.ctx.memorizedLayerLayout(subl)
	return &subLayout{
		id:     id,
		offset: offset,
		insnId: -1,
	}
}

func (ctx *compileContext) layoutQuery(l *layerLayout, s []int) *layoutQuery {
	q := &layoutQuery{
		l:      l,
		ctx:    ctx,
		varPos: make(map[int]int),
	}
	if l.sparse {
		for i, v := range l.placementSparse {
			if _, ok := q.varPos[s[v]]; ok {
				panic("unexpected situation")
			}
			q.varPos[s[v]] = i
		}
	} else {
		for i, v := range l.placementDense {
			if v != -1 {
				if _, ok := q.varPos[s[v]]; ok {
					panic("unexpected situation")
				}
				//fmt.Printf("/%d\n", v)
				q.varPos[s[v]] = i
			}
		}
	}
	return q
}

// connectWires solves the wire connection between two layers
func (ctx *compileContext) connectWires(a_, b_ int) int {
	mapId := int64(a_)<<32 | int64(b_)
	if v, ok := ctx.connectedWires[mapId]; ok {
		return v
	}
	a := ctx.layerLayout[a_]
	b := ctx.layerLayout[b_]
	if (a.layer+1 != b.layer && (a.layer != -1 || b.layer != -1)) || a.circuitId != b.circuitId {
		//fmt.Printf("%d %d %d %d\n", a.layer, b.layer, a.circuitId, b.circuitId)
		panic("unexpected situation")
	}
	ic := ctx.circuits[a.circuitId]
	circuit := ic.circuit
	curLayer := a.layer
	nextLayer := b.layer
	var curLc, nextLc *layerLayoutContext
	if curLayer >= 0 {
		curLc = &ic.lcs[curLayer]
		nextLc = &ic.lcs[nextLayer]
	} else {
		curLc = ic.lcHint
		nextLc = ic.lcHint
	}
	aq := ctx.layoutQuery(a, curLc.varIdx)
	bq := ctx.layoutQuery(b, nextLc.varIdx)

	//fmt.Printf("connectWires: %d %d circuitId=%d curLayer=%d\n", a_, b_, a.circuitId, curLayer)
	//fmt.Printf("curDense: %v\n", a.placementDense)
	//fmt.Printf("nextDense: %v\n", b.placementDense)
	//fmt.Printf("curVar: %v\n", curLc.varIdx)
	//fmt.Printf("nextVar: %v\n", nextLc.varIdx)

	// check if all variables exist in the layout
	for _, x := range curLc.varIdx {
		if _, ok := aq.varPos[x]; !ok {
			panic("unexpected situation")
		}
	}
	for _, x := range nextLc.varIdx {
		if _, ok := bq.varPos[x]; !ok {
			panic("unexpected situation")
		}
	}

	subInsnIds := make([]int, 0, len(ic.subCircuitInsnIds))
	subInsnMap := make(map[int]int)
	subCurLayout := make([]*subLayout, 0, len(ic.subCircuitInsnIds))
	subNextLayout := make([]*subLayout, 0, len(ic.subCircuitInsnIds))
	subCurLayoutAll := make(map[int]*subLayout)

	// find all sub circuits
	for i, insnId := range ic.subCircuitInsnIds {
		if !ic.isUsed[i+ic.nbVariable+ic.nbHintInputSub] {
			continue
		}
		insn := circuit.Instructions[insnId]
		subId := insn.SubCircuitId
		subC := ctx.circuits[subId]
		dep := subC.outputLayer
		inputLayer := ic.subCircuitStartLayer[i]
		outputLayer := inputLayer + dep
		var curLayout *subLayout = nil
		var nextLayout *subLayout = nil
		outf := func(x int) int {
			return subC.circuit.Output[x][0].VID0
		}
		hintf := func(x int) int {
			return subC.hintInputs[x]
		}
		if inputLayer <= curLayer && outputLayer >= nextLayer {
			// normal
			if inputLayer == curLayer {
				// for the input layer, we need to manually query the layout. (other layers are already subLayouts)
				vs := []int{}
				for _, x := range insn.Inputs {
					vs = append(vs, x[0].VID0)
				}
				vs = append(vs, ic.subCircuitHintInputs[i]...)
				curLayout = aq.query(vs, func(x int) int {
					if x < len(insn.Inputs) {
						return x + 1
					}
					return subC.hintInputs[x-len(insn.Inputs)]
				}, subId, 0)
			}
			if outputLayer == nextLayer {
				// also for the output layer
				nextLayout = bq.query(insn.OutputIds, outf, subId, dep)
			}
		} else if nextLayer != -1 && nextLayer <= inputLayer && len(ic.subCircuitHintInputs[i]) != 0 {
			// relay hint input
			if curLayer == 0 {
				curLayout = aq.query(ic.subCircuitHintInputs[i], hintf, subId, -1)
			}
			if nextLayer == inputLayer {
				nextLayout = bq.query(ic.subCircuitHintInputs[i], hintf, subId, -1)
				//fmt.Printf("hint next layout: %v %v\n", *nextLayout, ic.subCircuitHintInputs[i])
			}
		} else if curLayer == outputLayer {
			// it might be possible that some constraints are in the output layer, so we have to check it here
			curLayout = aq.query(insn.OutputIds, outf, subId, dep)
			subCurLayoutAll[insnId] = curLayout
			continue
		} else {
			continue
		}
		subInsnMap[insnId] = len(subInsnIds)
		subInsnIds = append(subInsnIds, insnId)
		subCurLayout = append(subCurLayout, curLayout)
		subNextLayout = append(subNextLayout, nextLayout)
	}

	// fill already known subLayouts
	for i := 0; i < len(a.subLayout); i++ {
		subCurLayout[subInsnMap[a.subLayout[i].insnId]] = &a.subLayout[i]
	}
	for i := 0; i < len(b.subLayout); i++ {
		subNextLayout[subInsnMap[b.subLayout[i].insnId]] = &b.subLayout[i]
	}

	res := &layered.Circuit{
		InputLen:    uint64(a.size),
		OutputLen:   uint64(b.size),
		SubCircuits: []layered.SubCircuit{},
		Mul:         []layered.GateMul{},
		Add:         []layered.GateAdd{},
		Cst:         []layered.GateCst{},
	}

	// connect sub circuits
	for i := 0; i < len(subInsnIds); i++ {
		subCurLayoutAll[subInsnIds[i]] = subCurLayout[i]
		//fmt.Printf("%d %d id=%d curLayout=%v nextLayout=%v\n", a_, b_, subInsnIds[i], subCurLayout[i], subNextLayout[i])
		//fmt.Printf("recursive: %d %d %d %d\n", subCurLayout[i].id, subNextLayout[i].id, subCurLayout[i].offset, subNextLayout[i].offset)
		scid := ctx.connectWires(subCurLayout[i].id, subNextLayout[i].id)
		al := layered.Allocation{
			InputOffset:  uint64(subCurLayout[i].offset),
			OutputOffset: uint64(subNextLayout[i].offset),
		}
		for j := 0; j <= len(res.SubCircuits); j++ {
			if j == len(res.SubCircuits) {
				res.SubCircuits = append(res.SubCircuits, layered.SubCircuit{
					Id:          uint64(scid),
					Allocations: []layered.Allocation{al},
				})
				break
			}
			if res.SubCircuits[j].Id == uint64(scid) {
				res.SubCircuits[j].Allocations = append(res.SubCircuits[j].Allocations, al)
				break
			}
		}
	}

	toBigInt := ctx.rc.Field.ToBigInt
	field := func() *big.Int {
		res := big.NewInt(0)
		res.Set(ctx.rc.Field.Field())
		return res
	}

	// connect self variables
	for _, x := range nextLc.varIdx {
		// only consider real variables, except it's hint relay layer
		if x >= ic.nbVariable && curLayer != -1 {
			continue
		}
		pos := bq.varPos[x]
		// if it's not the first layer, just relay it
		if ic.minLayer[x] != nextLayer || curLayer == -1 {
			//fmt.Printf("/relay %d: %d %d\n", x, aq.varPos[x], pos)
			res.Add = append(res.Add, layered.GateAdd{
				In:   uint64(aq.varPos[x]),
				Out:  uint64(pos),
				Coef: big.NewInt(1),
			})
			continue
		}
		e := ic.internalVariableExpr[x]
		for _, term := range e {
			//fmt.Printf("/%d %d %d\n", x, term.VID0, term.VID1)
			if term.VID0 == 0 {
				// constant
				res.Cst = append(res.Cst, layered.GateCst{
					Out:  uint64(pos),
					Coef: toBigInt(term.Coeff),
				})
			} else if term.VID1 == 0 {
				// add
				res.Add = append(res.Add, layered.GateAdd{
					In:   uint64(aq.varPos[term.VID0]),
					Out:  uint64(pos),
					Coef: toBigInt(term.Coeff),
				})
			} else {
				// mul
				res.Mul = append(res.Mul, layered.GateMul{
					In0:  uint64(aq.varPos[term.VID0]),
					In1:  uint64(aq.varPos[term.VID1]),
					Out:  uint64(pos),
					Coef: toBigInt(term.Coeff),
				})
			}
		}
	}
	// also combined output variables
	var cc *combinedConstraint = nil
	if nextLayer >= 0 {
		cc = ic.combinedConstraints[nextLayer]
	}
	if cc != nil {
		pos := bq.varPos[cc.id]
		for _, v := range cc.variables {
			//fmt.Printf("combine: %d %d\n", v, cc.id)
			var coef *big.Int
			if v >= ic.nbVariable {
				coef = big.NewInt(1)
			} else {
				coef = field()
			}
			res.Add = append(res.Add, layered.GateAdd{
				In:   uint64(aq.varPos[v]),
				Out:  uint64(pos),
				Coef: coef, // p means random
			})
		}
		for j, i := range cc.subCircuitIds {
			if !ic.isUsed[j+ic.nbVariable+ic.nbHintInputSub] {
				continue
			}
			insnId := ic.subCircuitInsnIds[i]
			insn := circuit.Instructions[insnId]
			inputLayer := ic.subCircuitStartLayer[i]
			vid := ctx.circuits[insn.SubCircuitId].combinedConstraints[curLayer-inputLayer].id
			vpid := ctx.circuits[insn.SubCircuitId].lcs[curLayer-inputLayer].varMap[vid]
			//fmt.Printf("vid: %d vpid: %d\n", vid, vpid)
			layout := ctx.layerLayout[subCurLayoutAll[insnId].id]
			//fmt.Printf("layout: %v\n", layout)
			spos := -1
			if layout.sparse {
				for i, v := range layout.placementSparse {
					if v == vpid {
						spos = i
						break
					}
				}
			} else {
				for i, v := range layout.placementDense {
					if v == vpid {
						spos = i
						break
					}
				}
			}
			if pos == -1 {
				panic("unexpected situation")
			}
			res.Add = append(res.Add, layered.GateAdd{
				In:   uint64(subCurLayoutAll[insnId].offset + spos),
				Out:  uint64(pos),
				Coef: toBigInt(ctx.rc.Field.One()),
			})
		}
	}

	resId := len(ctx.compiledCircuits)
	ctx.compiledCircuits = append(ctx.compiledCircuits, res)
	ctx.connectedWires[mapId] = resId

	//fmt.Printf("connected (%d, %d) -> %d\n", a_, b_, resId)

	return resId
}
