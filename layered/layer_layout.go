package layered

import (
	"fmt"
	"sort"

	"github.com/Zklib/gkr-compiler/circuitir"
	"github.com/Zklib/gkr-compiler/utils"
)

type layerLayoutContext struct {
	varIdx            []int       // global index of variables occurring in this layer
	varMap            map[int]int // inverse of varIdx
	prevCircuitInsnId map[int]int // insn id of previous circuit
	prevCircuitNbOut  map[int]int // number of outputs of previous circuit, used to check if all output variables are used
	placement         map[int]int // placement group of each variable
	parent            []int       // parent placement group of some placement group
	req               placementReqList

	middleSubCircuits []int // sub-circuits who have middle layers in this layer (referenced by index in subCircuitInsnIds)
}

type placementReqList []placementReq

// we will sort placement requests by size, and then greedy
type placementReq struct {
	insnId    int
	inputIds  []int
	inputSize int
}

func (e placementReqList) Len() int {
	return len(e)
}
func (e placementReqList) Swap(i, j int) {
	e[i], e[j] = e[j], e[i]
}

func (e placementReqList) Less(i, j int) bool {
	if e[i].inputSize != e[j].inputSize {
		return e[i].inputSize > e[j].inputSize
	}
	return e[i].insnId < e[j].insnId
}

func (ctx *compileContext) prepareLayerLayoutContext(ic *circuitIrContext) {
	// find out the variables in each layer
	ic.lcs = make([]layerLayoutContext, ic.outputLayer+1)
	ic.lcHint = new(layerLayoutContext)
	for i := 0; i < ic.nbVariable; i++ {
		if ic.isUsed[i] {
			for j := ic.minLayer[i]; j <= ic.maxLayer[i]; j++ {
				ic.lcs[j].varIdx = append(ic.lcs[j].varIdx, i)
			}
		}
	}
	for i := 0; i < len(ic.subCircuitInsnIds); i++ {
		inputLayer := ic.subCircuitStartLayer[i]
		for _, x := range ic.subCircuitHintInputs[i] {
			fmt.Printf("hint enqueue %d (inputLayer=%d)\n", x, inputLayer)
			ic.lcs[0].varIdx = append(ic.lcs[0].varIdx, x)
			if inputLayer != 0 {
				ic.lcs[inputLayer].varIdx = append(ic.lcs[inputLayer].varIdx, x)
			}
		}
	}
	for i := 0; i <= ic.outputLayer; i++ {
		if ic.combinedConstraints[i] != nil {
			ic.lcs[i].varIdx = append(ic.lcs[i].varIdx, ic.combinedConstraints[i].id)
		}
	}

	for i := 0; i <= ic.outputLayer; i++ {
		ic.lcs[i].varMap = make(map[int]int)
		ic.lcs[i].prevCircuitInsnId = make(map[int]int)
		ic.lcs[i].prevCircuitNbOut = make(map[int]int)
		ic.lcs[i].placement = make(map[int]int)
		for j, x := range ic.lcs[i].varIdx {
			ic.lcs[i].varMap[x] = j
		}
	}

	// prepare lcHint
	for _, insn := range ic.circuit.Instructions {
		if insn.Type == circuitir.IHint {
			ic.lcHint.varIdx = append(ic.lcHint.varIdx, insn.OutputIds...)
		}
	}
	ic.lcHint.varMap = make(map[int]int)
	for j, x := range ic.lcHint.varIdx {
		ic.lcHint.varMap[x] = j
	}

	// for each sub-circuit, enqueue the placement request in input layer, and mark prevCircuitInsnId in output layer
	// also push all middle layers to the layer context
	for i, insnId := range ic.subCircuitInsnIds {
		insn := ic.circuit.Instructions[insnId]
		inputLayer := ic.subCircuitStartLayer[i]
		outputLayer := ctx.circuits[insn.SubCircuitId].outputLayer + inputLayer
		inputIds := make([]int, len(insn.Inputs))
		for j, x := range insn.Inputs {
			inputIds[j] = x[0].VID0
		}
		ic.lcs[inputLayer].req = append(ic.lcs[inputLayer].req, placementReq{insnId, inputIds, len(insn.Inputs)})

		for _, x := range insn.OutputIds {
			ic.lcs[outputLayer].prevCircuitInsnId[x] = insnId
		}
		ic.lcs[outputLayer].prevCircuitNbOut[insnId] = len(insn.OutputIds)

		// hint input is also considered as output of some relay circuit
		if len(ic.subCircuitHintInputs[i]) != 0 {
			for _, x := range ic.subCircuitHintInputs[i] {
				ic.lcs[inputLayer].prevCircuitInsnId[x] = insnId + len(ic.circuit.Instructions)
			}
			ic.lcs[inputLayer].prevCircuitNbOut[insnId+len(ic.circuit.Instructions)] = len(ic.subCircuitHintInputs[i])
			for j := 1; j < inputLayer; j++ {
				ic.lcs[j].middleSubCircuits = append(ic.lcs[j].middleSubCircuits, i)
			}
		}
		for j := inputLayer + 1; j < outputLayer; j++ {
			ic.lcs[j].middleSubCircuits = append(ic.lcs[j].middleSubCircuits, i)
		}
	}

	for i := 0; i <= ic.outputLayer; i++ {
		lc := &ic.lcs[i]
		for _, x := range lc.varIdx {
			lc.placement[x] = 0
		}
		lc.parent = []int{0}
		sort.Sort(lc.req)
		// greedy placement
		for _, req := range lc.req {
			pcCnt := make(map[int]int) // prev circuit count
			plCnt := make(map[int]int) // placement count
			for _, x := range req.inputIds {
				if pc, ok := lc.prevCircuitInsnId[x]; ok {
					pcCnt[pc] = 0
				}
				plCnt[lc.placement[x]] = 0
			}
			for _, x := range req.inputIds {
				if pc, ok := lc.prevCircuitInsnId[x]; ok {
					pcCnt[pc] += 1
				}
				plCnt[lc.placement[x]] += 1
			}
			// if all inputs don't split previout circuits, and they are in the same placement group,
			// we can create a new placement group containing them
			flag := len(plCnt) == 1
			for k, v := range pcCnt {
				if v != lc.prevCircuitNbOut[k] {
					flag = false
				}
			}
			if flag {
				np := len(lc.parent) // new placement group id
				for _, x := range req.inputIds {
					lc.placement[x] = np
				}
				parent := 0
				for x := range plCnt {
					parent = x
				}
				lc.parent = append(lc.parent, parent)
			}
		}
		// TODO: partial merge
	}
}

// TODO: use better data structure to maintain the segments

// finalized layout of a layer
// dense -> placementDense[i] = variable on slot i (placementDense[i] == j means i-th slot stores varIdx[j])
// sparse -> placementSparse[i] = variable on slot i, and there are subLayouts.
type layerLayout struct {
	circuitId uint64
	layer     int

	sparse          bool
	size            int
	placementDense  []int
	placementSparse map[int]int
	subLayout       []subLayout
}

// make a copy of the layout for substitution
func (l *layerLayout) CopyForSubs() *layerLayout {
	if l.sparse {
		panic("unexpected situation")
	}
	return &layerLayout{
		circuitId:      ^uint64(0),
		layer:          -1,
		sparse:         false,
		size:           l.size,
		placementDense: append([]int(nil), l.placementDense...),
	}
}

func (l *layerLayout) SubsArray(s []int) {
	if l.sparse {
		panic("unexpected situation")
	}
	for i := 0; i < l.size; i++ {
		if l.placementDense[i] != -1 {
			l.placementDense[i] = s[l.placementDense[i]]
		}
	}
}

func (l *layerLayout) SubsMap(s map[int]int) {
	if l.sparse {
		panic("unexpected situation")
	}
	for i := 0; i < l.size; i++ {
		if l.placementDense[i] != -1 {
			l.placementDense[i] = s[l.placementDense[i]]
		}
	}
}

func (l *layerLayout) HashCode() uint64 {
	const p1 = 1000000007
	const p2 = 1000000009
	var res uint64 = l.circuitId ^ uint64(l.layer)
	if l.sparse {
		for k, v := range l.placementSparse {
			res = (res*p1+uint64(k))*p1 + uint64(v)
		}
		for _, v := range l.subLayout {
			res = ((res*p1+uint64(v.id))*p1+uint64(v.offset))*p1 + uint64(v.insnId)
		}
	} else {
		for _, v := range l.placementDense {
			res = (res*p2 + uint64(v))
		}
	}
	return res
}

func (l *layerLayout) EqualI(e utils.Hashable) bool {
	other := e.(*layerLayout)
	if l.circuitId != other.circuitId || l.layer != other.layer || l.sparse != other.sparse || l.size != other.size {
		return false
	}
	if l.sparse {
		if len(l.placementSparse) != len(other.placementSparse) {
			return false
		}
		for k, v := range l.placementSparse {
			if v2, ok := other.placementSparse[k]; !ok || v != v2 {
				return false
			}
		}
		if len(l.subLayout) != len(other.subLayout) {
			return false
		}
		for i, v := range l.subLayout {
			if v != other.subLayout[i] {
				return false
			}
		}
	} else {
		if len(l.placementDense) != len(other.placementDense) {
			return false
		}
		for i, v := range l.placementDense {
			if v != other.placementDense[i] {
				return false
			}
		}
	}
	return true
}

type subLayout struct {
	id     int // unique layout id in a compile context
	offset int // offset in layout
	insnId int // instruction id corresponding to this sub-layout
}

// request for layer layout
type layerReq struct {
	circuitId uint64
	layer     int // which layer to solve?

	// TODO: more requirements, e.g. alignment
}

func (l *layerReq) HashCode() uint64 {
	return l.circuitId ^ uint64(l.layer)
}

func (l *layerReq) EqualI(e utils.Hashable) bool {
	other := e.(*layerReq)
	if l.circuitId != other.circuitId || l.layer != other.layer {
		return false
	}
	return true
}

func (ctx *compileContext) memorizedLayerLayout(layout *layerLayout) int {
	nid := len(ctx.layerLayout)
	nid = ctx.layerLayoutMap.Add(layout, nid).(int)
	fmt.Printf("[[[%d %d]]]\n", nid, layout.HashCode())
	if nid == len(ctx.layerLayout) {
		ctx.layerLayout = append(ctx.layerLayout, layout)
	}
	return nid
}

func (ctx *compileContext) solveLayerLayout(req *layerReq) int {
	id, ok := ctx.layerReqToLayout.Find(req)
	if ok {
		return id.(int)
	}

	ic := ctx.circuits[req.circuitId]
	var res *layerLayout

	if req.layer >= 0 {
		res = ctx.solveLayerLayoutNormal(ic, req)
	} else {
		res = ctx.solveLayerLayoutHintRelay(ic, req)
	}
	nid := ctx.memorizedLayerLayout(res)
	ctx.layerReqToLayout.Set(req, nid)
	return nid
}

func (ctx *compileContext) mergeLayouts(s [][]int, additional []int) []int {
	// currently it's a simple greedy algorithm
	// sort groups by size, and then place them one by one
	// since their size are always 2^n, the result is aligned
	// finally we insert the remaining variables to the empty slots
	// TODO: improve this
	n := 0
	for _, x := range s {
		m := len(x)
		n += m
		if (m & -m) != m {
			panic("unexpected situation: placement group size should be power of 2")
		}
	}
	n = utils.NextPowerOfTwo(n, false)
	res := make([]int, 0, n)

	order := make([]int, 0, len(s))
	for i, x := range s {
		if len(x) != 0 {
			order = append(order, i)
		}
	}
	utils.SortIntSeq(order, func(i, j int) bool {
		return len(s[i]) > len(s[j])
	})

	for _, x_ := range order {
		pg := s[x_]
		if len(res)%len(pg) != 0 {
			panic("unexpected situation")
		}
		placed := false
		// TODO: better collision detection
		for i := 0; i < len(res); i += len(pg) {
			ok := true
			for j, x := range pg {
				if res[i+j] != -1 && x != -1 {
					ok = false
					break
				}
			}
			if ok {
				for j, x := range pg {
					res[i+j] = x
				}
				placed = true
				break
			}
		}
		if !placed {
			res = append(res, pg...)
		}
	}

	slot := 0
	for _, x := range additional {
		for slot < len(res) && res[slot] != -1 {
			slot++
		}
		if slot >= len(res) {
			res = append(res, x)
		} else {
			res[slot] = x
		}
	}

	pad := utils.NextPowerOfTwo(len(res), false) - len(res)
	for i := 0; i < pad; i++ {
		res = append(res, -1)
	}

	return res
}

func (ctx *compileContext) solveLayerLayoutHintRelay(ic *circuitIrContext, req *layerReq) *layerLayout {
	s := make([]int, len(ic.lcHint.varIdx))
	for i := 0; i < len(s); i++ {
		s[i] = i
	}
	placement := ctx.mergeLayouts(nil, s)
	return &layerLayout{
		circuitId:      req.circuitId,
		layer:          -1,
		sparse:         false,
		size:           len(placement),
		placementDense: placement,
	}
}

func (ctx *compileContext) solveLayerLayoutNormal(ic *circuitIrContext, req *layerReq) *layerLayout {
	lc := &ic.lcs[req.layer]

	// first iterate prev layer circuits, and solve their output layout
	layouts := make(map[int]*layerLayout)
	for x_ := range lc.prevCircuitNbOut {
		var subLayer int
		var insn *circuitir.Instruction
		x := x_
		if x >= len(ic.circuit.Instructions) {
			x -= len(ic.circuit.Instructions)
			insn = &ic.circuit.Instructions[x]
			subLayer = -1
		} else {
			insn = &ic.circuit.Instructions[x]
			subLayer = ctx.circuits[insn.SubCircuitId].outputLayer
		}
		layoutId := ctx.solveLayerLayout(&layerReq{
			circuitId: insn.SubCircuitId,
			layer:     subLayer,
		})
		// convert id to local id
		layout := ctx.layerLayout[layoutId].CopyForSubs()
		fmt.Printf("layout %v sublayer %d\n", layout.placementDense, subLayer)
		if subLayer >= 0 {
			layout.SubsArray(ctx.circuits[insn.SubCircuitId].lcs[subLayer].varIdx)
			fmt.Printf("layout %v %v\n", layout.placementDense, ctx.circuits[insn.SubCircuitId].outputOrder)
			layout.SubsMap(ctx.circuits[insn.SubCircuitId].outputOrder)
			fmt.Printf("layout %v\n", layout.placementDense)
			layout.SubsArray(insn.OutputIds)
			fmt.Printf("layout %v\n", layout.placementDense)
		} else {
			layout.SubsArray(ctx.circuits[insn.SubCircuitId].lcHint.varIdx)
			layout.SubsMap(ctx.circuits[insn.SubCircuitId].hintInputsMap)
			layout.SubsArray(ic.subCircuitHintInputs[ic.subCircuitLocMap[x]])
		}
		layout.SubsMap(lc.varMap)
		fmt.Printf("layout after subs %v\n", layout.placementDense)
		layouts[x] = layout
	}

	// build the tree of placement groups
	childrenVariables := make([][]int, len(lc.parent))
	for i, x := range lc.varIdx {
		if _, ok := lc.prevCircuitInsnId[x]; !ok {
			childrenVariables[lc.placement[x]] = append(childrenVariables[lc.placement[x]], i)
		}
	}
	childrenPrevCircuits := make([][]*layerLayout, len(lc.parent))
	for x, layout := range layouts {
		v := ic.circuit.Instructions[x].OutputIds[0]
		childrenPrevCircuits[lc.placement[v]] = append(childrenPrevCircuits[lc.placement[v]], layout)
	}
	childrenNodes := make([][]int, len(lc.parent))
	for i, x := range lc.parent {
		if i == 0 {
			continue
		}
		childrenNodes[x] = append(childrenNodes[x], i)
	}
	fmt.Printf("============================================= childrenNodes: %v\n", childrenNodes)
	placements := make([][]int, len(lc.parent))
	for i := len(lc.parent) - 1; i >= 0; i-- {
		s := [][]int{}
		for _, x := range childrenNodes[i] {
			s = append(s, placements[x])
		}
		for _, x := range childrenPrevCircuits[i] {
			s = append(s, x.placementDense)
		}
		placements[i] = ctx.mergeLayouts(s, childrenVariables[i])
		fmt.Printf("%d %v %v\n", i, placements[i], childrenVariables[i])
	}

	// now placements[0] contains all direct variables
	// we only need to merge with middle layers
	// currently it's the most basic merging algorithm - just put them together
	// TODO: optimize the merging algorithm

	if len(lc.middleSubCircuits) == 0 {
		return &layerLayout{
			circuitId:      req.circuitId,
			layer:          req.layer,
			sparse:         false,
			size:           len(placements[0]),
			placementDense: placements[0],
		}
	}

	middleLayouts := make([]int, len(lc.middleSubCircuits))
	for i, id := range lc.middleSubCircuits {
		startLayer := ic.subCircuitStartLayer[id]
		reqLayer := req.layer - startLayer
		if req.layer < startLayer {
			// hint input relay layer
			reqLayer = -1
		}
		middleLayouts[i] = ctx.solveLayerLayout(&layerReq{
			circuitId: ic.circuit.Instructions[ic.subCircuitInsnIds[id]].SubCircuitId,
			layer:     reqLayer,
		})
	}
	sizes := make([]int, len(middleLayouts)+1)
	sizes[0] = len(placements[0])
	for i, x := range middleLayouts {
		sizes[i+1] = ctx.layerLayout[x].size
	}
	order := make([]int, len(sizes))
	for i := 0; i < len(sizes); i++ {
		order[i] = i
	}
	utils.SortIntSeq(order, func(i, j int) bool {
		return sizes[i] > sizes[j]
	})
	cur := 0
	res := &layerLayout{
		circuitId:       req.circuitId,
		layer:           req.layer,
		sparse:          true,
		placementSparse: make(map[int]int),
	}
	for _, i := range order {
		if i == 0 {
			flag := false
			for j, x := range placements[0] {
				if x != -1 {
					flag = true
					res.placementSparse[cur+j] = x
				}
			}
			if !flag {
				continue
			}
		} else {
			res.subLayout = append(res.subLayout, subLayout{
				id:     middleLayouts[i-1],
				offset: cur,
				insnId: ic.subCircuitInsnIds[lc.middleSubCircuits[i-1]],
			})
		}
		cur += sizes[i]
	}
	res.size = utils.NextPowerOfTwo(cur, false)

	return res
}
