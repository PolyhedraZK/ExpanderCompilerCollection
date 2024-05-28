// Package layering provides functionality to compile an IR of a circuit into a layered circuit.
package layering

import (
	"github.com/Zklib/gkr-compiler/expr"
	"github.com/Zklib/gkr-compiler/ir"
	"github.com/Zklib/gkr-compiler/layered"
	"github.com/Zklib/gkr-compiler/utils"
)

type compileContext struct {
	// the root circuit
	rc *ir.RootCircuit

	// for each circuit ir, we need a context to store some intermediate information
	circuits map[uint64]*irContext

	// topo-sorted order
	order []uint64

	// all generated layer layouts
	layerLayoutMap utils.Map
	layerLayout    []*layerLayout
	// map from layerReq to layerLayout id
	layerReqToLayout utils.Map

	// compiled layered circuits
	compiledCircuits []*layered.Circuit
	connectedWires   map[int64]int

	// layout id of each layer
	layoutIds []int
	// compiled circuit id of each layer
	layers []int

	// input order
	inputOrder ir.InputOrder
}

type irContext struct {
	circuit        *ir.Circuit
	nbVariable     int // number of variables in the circuit
	nbSubCircuits  int // number of sub circuits
	nbHintInput    int // number of hint inputs in the circuit itself
	nbHintInputSub int // number of hint inputs in sub circuits (these must be propagated from the global input)

	// for each variable, we need to find the min and max layer it should exist.
	// we assume input layer = 0, and output layer is at least 1
	// it includes only variables mentioned in instructions, so internal variables in sub circuits are ignored here.
	minLayer    []int
	maxLayer    []int
	outputLayer int
	isUsed      []bool

	outputOrder map[int]int // outputOrder[x] == y -> x is the y-th output

	subCircuitLocMap     map[int]int
	subCircuitInsnIds    []int
	subCircuitHintInputs [][]int
	subCircuitStartLayer []int

	hintInputs    []int // hint inputs variable id of the circuit itself
	hintInputsMap map[int]int

	// combined constraints of each layer
	combinedConstraints []*combinedConstraint

	internalVariableExpr map[int]expr.Expression
	isRandomVariable     map[int]bool

	// layer layout contexts
	lcs    []layerLayoutContext
	lcHint *layerLayoutContext // hint relayer
}

type combinedConstraint struct {
	// id of this combined variable
	id int
	// id of combined variables
	variables []int
	// id of sub circuits (it will combine their combined constraints)
	// if a sub circuit has a combined output in this layer, it must be unique. So circuit id is sufficient.
	// = {x} means subCircuitInsnIds[x]
	subCircuitIds []int
}

// Compile takes an IR RootCircuit and compiles it into a layered circuit.
func Compile(rc *ir.RootCircuit) (*layered.RootCircuit, *ir.InputOrder) {
	ctx := newCompileContext(rc)
	ctx.compile()
	layersUint64 := make([]uint64, len(ctx.layers))
	for i, x := range ctx.layers {
		layersUint64[i] = uint64(x)
	}
	return &layered.RootCircuit{
		Circuits: ctx.compiledCircuits,
		Layers:   layersUint64,
		Field:    rc.Field.Field(),
	}, &ctx.inputOrder
}

// ProfilingCompile is similar to Compile but is used for profiling purposes.
// It partially compiles the circuit and computes cost associated with each variable in the circuit.
func ProfilingCompile(rc *ir.RootCircuit) []int {
	ctx := newCompileContext(rc)
	ctx.compile()

	ic := ctx.circuits[0]
	res := make([]int, len(ic.minLayer))
	for i := range ic.minLayer {
		if ic.isUsed[i] {
			res[i] = ic.maxLayer[i] - ic.minLayer[i] + 1
		}
	}
	return res
}

func newCompileContext(rc *ir.RootCircuit) *compileContext {
	return &compileContext{
		rc:               rc,
		circuits:         make(map[uint64]*irContext),
		layerLayoutMap:   make(utils.Map),
		layerReqToLayout: make(utils.Map),
		connectedWires:   make(map[int64]int),
	}
}

func (ctx *compileContext) compile() {
	// 1. do a toposort of the circuits
	ctx.dfsTopoSort(0)

	// 2. compute min and max layers for each circuit
	for _, id := range ctx.order {
		ctx.computeMinMaxLayers(ctx.circuits[id])
	}

	// 3. prepare layer layout contexts
	for _, id := range ctx.order {
		ctx.prepareLayerLayoutContext(ctx.circuits[id])
	}

	// 4. solve layer layout for root circuit (it also recursively solves all requires sub-circuits)
	ctx.layoutIds = make([]int, ctx.circuits[0].outputLayer+1)
	for i := 0; i <= ctx.circuits[0].outputLayer; i++ {
		ctx.layoutIds[i] = ctx.solveLayerLayout(&layerReq{
			circuitId: 0,
			layer:     i,
		})
	}

	// 5. generate wires
	ctx.layers = make([]int, ctx.circuits[0].outputLayer)
	for i := 0; i < ctx.circuits[0].outputLayer; i++ {
		ctx.layers[i] = ctx.connectWires(ctx.layoutIds[i], ctx.layoutIds[i+1])
	}

	// 6. record the input order (used to generate witness)
	ctx.inputOrder = ctx.recordInputOrder(ctx.layoutIds[0])
}

// toposort dfs
func (ctx *compileContext) dfsTopoSort(id uint64) {
	if _, ok := ctx.circuits[id]; ok {
		return
	}

	nv := 0
	ns := 0
	nh := 0
	nhs := 0
	circuit := ctx.rc.Circuits[id]
	nv = circuit.NbExternalInput
	for _, insn := range circuit.Instructions {
		if insn.Type == ir.ISubCircuit {
			ctx.dfsTopoSort(insn.SubCircuitId)
			ns += 1
			nhs += ctx.circuits[insn.SubCircuitId].nbHintInput + ctx.circuits[insn.SubCircuitId].nbHintInputSub
		} else if insn.Type == ir.IHint {
			nh += len(insn.OutputIds)
		}
		for _, x := range insn.OutputIds {
			if x > nv {
				nv = x
			}
		}
	}
	// nv is currently the maximum id of varaibles, so we need to add 1 to get the count
	nv += 1

	// when all children are done, we enqueue the current circuit
	ctx.order = append(ctx.order, id)
	ctx.circuits[id] = &irContext{
		circuit:              circuit,
		nbVariable:           nv,
		nbSubCircuits:        ns,
		nbHintInput:          nh,
		nbHintInputSub:       nhs,
		subCircuitLocMap:     make(map[int]int),
		outputOrder:          make(map[int]int),
		hintInputsMap:        make(map[int]int),
		internalVariableExpr: make(map[int]expr.Expression),
		isRandomVariable:     make(map[int]bool),
	}
}

func (ctx *compileContext) isSingleVariable(e expr.Expression) bool {
	return len(e) == 1 && e[0].VID1 == 0 && e[0].VID0 != 0 && ctx.rc.Field.IsOne(e[0].Coeff)
}

func (ctx *compileContext) computeMinMaxLayers(ic *irContext) {
	// variables
	// 0..nbVariable: normal variables
	// nbVariable..nbVariable+nbHintInputSub: hint inputs. root circuit first, and then sub circuits by insn order
	// next nbSubCircuits terms: sub circuit virtual variables (in order to lower the number of edges)
	// next ? terms: random sum of constraints
	nv := ic.nbVariable
	ns := ic.nbSubCircuits
	nh := ic.nbHintInput
	nhs := ic.nbHintInputSub
	n := nv + nhs + ns
	circuit := ic.circuit

	preAllocSize := n
	if n < 1000 {
		preAllocSize += n
	} else {
		preAllocSize += 1000
	}
	ic.minLayer = make([]int, n, preAllocSize)
	ic.maxLayer = make([]int, n, preAllocSize)
	for i := 0; i < n; i++ {
		ic.minLayer[i] = -1
	}
	for i := 0; i < circuit.NbExternalInput+1; i++ {
		ic.minLayer[i] = 0
	}

	// layer advanced by each variable.
	// for normal variable, it's 1
	// for sub circuit virtual variable, it's output layer - 1
	layerAdvance := make([]int, n, preAllocSize)

	inEdges := make([][]int, n)  // inEdges[i] = {j} means j -> i
	outEdges := make([][]int, n) // outEdges[i] = {j} means i -> j
	addEdge := func(x, y int) {  // add edge x -> y
		inEdges[y] = append(inEdges[y], x)
		outEdges[x] = append(outEdges[x], y)
	}

	ic.subCircuitInsnIds = make([]int, 0, ns)
	ic.subCircuitHintInputs = make([][]int, 0, ns)

	ic.hintInputs = make([]int, 0, nh+nhs)

	// get all input wires and build the graph
	// also computes the topo order
	q0 := make([]int, 0, preAllocSize) // input
	q1 := make([]int, 0, preAllocSize) // other
	for i := 1; i <= circuit.NbExternalInput; i++ {
		q0 = append(q0, i)
	}
	hintInputSubIdx := nv
	for i, insn := range circuit.Instructions {
		if insn.Type == ir.IInternalVariable {
			e := insn.Inputs[0]
			usedVar := make(map[int]bool)
			for _, term := range e {
				if term.Coeff.IsZero() {
					continue
				}
				if term.VID0 != 0 {
					usedVar[term.VID0] = true
				}
				if term.VID1 != 0 {
					usedVar[term.VID1] = true
				}
			}
			y := insn.OutputIds[0]
			for x := range usedVar {
				//fmt.Printf("%d %d %d %d\n", i, x, y, n)
				addEdge(x, y)
			}
			q1 = append(q1, y)
			layerAdvance[y] = 1
			ic.internalVariableExpr[y] = e
			if len(usedVar) == 0 {
				// actually this only happens at output layer
				ic.minLayer[y] = 1
			}
		} else if insn.Type == ir.IHint {
			for _, x := range insn.OutputIds {
				ic.minLayer[x] = 0
				q0 = append(q0, x)
				ic.hintInputs = append(ic.hintInputs, x)
			}
		} else if insn.Type == ir.ISubCircuit {
			// check if every input is single variable, and add edges
			k := len(ic.subCircuitInsnIds) + nv + nhs
			for _, x := range insn.Inputs {
				if !ctx.isSingleVariable(x) {
					panic("subcircuit input should be a single variable")
				}
				addEdge(x[0].VID0, k)
			}
			subh := ctx.circuits[insn.SubCircuitId].nbHintInput + ctx.circuits[insn.SubCircuitId].nbHintInputSub
			subhs := []int{}
			for j := 0; j < subh; j++ {
				addEdge(hintInputSubIdx+j, k)
				q0 = append(q0, hintInputSubIdx+j)
				subhs = append(subhs, hintInputSubIdx+j)
				ic.minLayer[hintInputSubIdx+j] = 0
			}
			hintInputSubIdx += subh

			q1 = append(q1, k)
			layerAdvance[k] = ctx.circuits[insn.SubCircuitId].outputLayer - 1
			for _, y := range insn.OutputIds {
				addEdge(k, y)
				q1 = append(q1, y)
				layerAdvance[y] = 1
			}
			ic.subCircuitInsnIds = append(ic.subCircuitInsnIds, i)
			ic.subCircuitHintInputs = append(ic.subCircuitHintInputs, subhs)
		} else if insn.Type == ir.IGetRandom {
			for _, x := range insn.OutputIds {
				// minLayer is 1, since we can't get a random value at input layer
				ic.minLayer[x] = 1
				q0 = append(q0, x)
				ic.isRandomVariable[x] = true
			}
		}
	}
	q0 = append(q0, q1...) // the merged topo order
	//fmt.Printf("{%v}\n", q0)

	for i := 0; i < nhs; i++ {
		ic.hintInputs = append(ic.hintInputs, nv+i)
	}
	for i, v := range ic.hintInputs {
		ic.hintInputsMap[v] = i
	}

	// bfs from output wire and constraints
	ic.isUsed = make([]bool, n, preAllocSize)
	q1 = q1[:0]
	setUsed := func(x int) {
		if !ic.isUsed[x] {
			ic.isUsed[x] = true
			q1 = append(q1, x)
		}
	}
	for _, e := range circuit.Output {
		if !ctx.isSingleVariable(e) {
			panic("output should be a single variable")
		}
		setUsed(e[0].VID0)
	}
	for _, e := range circuit.Constraints {
		if !ctx.isSingleVariable(e) {
			panic("constraint should be a single variable")
		}
		setUsed(e[0].VID0)
	}
	// if some constraint is set in a sub circuit, we need to mark the full sub circuit used
	for i, x := range ic.subCircuitInsnIds {
		for _, y := range ctx.circuits[circuit.Instructions[x].SubCircuitId].combinedConstraints {
			if y != nil {
				setUsed(nv + nhs + i)
				for _, z := range outEdges[nv+nhs+i] {
					setUsed(z)
				}
				break
			}
		}
	}
	// bfs
	for i := 0; i < len(q1); i++ {
		y := q1[i]
		for _, x := range inEdges[y] {
			setUsed(x)
		}
	}
	// if an output is used, mark all as used
	for i := range ic.subCircuitInsnIds {
		for _, y := range outEdges[nv+nhs+i] {
			if ic.isUsed[y] {
				for _, z := range outEdges[nv+nhs+i] {
					setUsed(z)
				}
				break
			}
		}
	}

	// filter out unused variables in the queue
	q1 = q1[:0]
	for _, x := range q0 {
		if ic.isUsed[x] {
			q1 = append(q1, x)
		}
	}
	q := q1

	//fmt.Printf("{%v}\n", q)

	// compute the min layer (depth) of each variable
	for _, x := range q {
		for _, y := range outEdges[x] {
			if ic.isUsed[y] {
				if ic.minLayer[y] < ic.minLayer[x]+layerAdvance[y] {
					ic.minLayer[y] = ic.minLayer[x] + layerAdvance[y]
				}
			}
		}
	}

	// compute sub circuit start layer
	ic.subCircuitStartLayer = make([]int, ns)
	for i := 0; i < len(ic.subCircuitInsnIds); i++ {
		if ic.isUsed[nv+nhs+i] {
			ic.subCircuitStartLayer[i] = ic.minLayer[nv+nhs+i] - layerAdvance[nv+nhs+i]
		} else {
			ic.subCircuitStartLayer[i] = -1
		}
	}

	// compute output layer and order
	ic.outputLayer = -1
	for i, x := range circuit.Output {
		if ic.outputLayer < ic.minLayer[x[0].VID0] {
			ic.outputLayer = ic.minLayer[x[0].VID0]
		}
		ic.outputOrder[x[0].VID0] = i
	}

	// add combined constraints variables, and also update output layer
	maxOccuredLayer := 0
	for i := 0; i < len(ic.minLayer); i++ {
		if ic.minLayer[i] > maxOccuredLayer {
			maxOccuredLayer = ic.minLayer[i]
		}
	}
	cc := make([]*combinedConstraint, maxOccuredLayer+3)
	for i := 0; i < len(cc); i++ {
		cc[i] = &combinedConstraint{}
	}
	for _, x := range circuit.Constraints {
		xid := x[0].VID0
		xl := ic.minLayer[xid] + 1
		cc[xl].variables = append(cc[xl].variables, xid)
	}
	for i, subId := range ic.subCircuitInsnIds {
		if !ic.isUsed[nv+nhs+i] {
			continue
		}
		subCircuit := ctx.circuits[circuit.Instructions[subId].SubCircuitId]
		for j, x := range subCircuit.combinedConstraints {
			if x != nil {
				sl := j + ic.subCircuitStartLayer[i] + 1
				cc[sl].subCircuitIds = append(cc[sl].subCircuitIds, i)
			}
		}
	}
	// special check: if this is the root circuit, we will merge them into one
	if ic == ctx.circuits[0] {
		first := 0
		for first < len(cc) && len(cc[first].variables) == 0 && len(cc[first].subCircuitIds) == 0 {
			first++
		}
		if first == len(cc) {
			panic("no constraints in the root circuit")
		}
		last := maxOccuredLayer + 1
		for i := first + 1; i <= last; i++ {
			// these ids should be layer-first+n
			cc[i].variables = append(cc[i].variables, i-1-first+n)
		}
		cc = cc[:last+1]
	}
	for i := 0; i < len(cc); i++ {
		if len(cc[i].variables) > 0 || len(cc[i].subCircuitIds) > 0 {
			cc[i].id = n
			if i+1 > ic.outputLayer {
				// currently the implementation doesn't allow contraint variable in the output layer
				// so we have to add 1 in non-root circuits
				if ic == ctx.circuits[0] {
					ic.outputLayer = i
				} else {
					ic.outputLayer = i + 1
				}
			}
			// we don't need to add edges here, since they will be never used below
			ic.minLayer = append(ic.minLayer, i)
			ic.maxLayer = append(ic.maxLayer, i)
			ic.isUsed = append(ic.isUsed, true)
			outEdges = append(outEdges, []int{})
			q = append(q, n)
			n++
		} else {
			cc[i] = nil
		}
	}
	ic.combinedConstraints = cc
	if ic == ctx.circuits[0] {
		if ic.outputLayer+1 <= len(cc) {
			ic.outputLayer = len(cc) - 1
		} else {
			panic("unexpected situation")
		}
	}

	// compute maxLayer
	for i := 0; i < n; i++ {
		ic.maxLayer[i] = ic.minLayer[i]
	}
	for _, x := range q {
		for _, y := range outEdges[x] {
			if ic.isUsed[y] && ic.minLayer[y]-layerAdvance[y] > ic.maxLayer[x] {
				ic.maxLayer[x] = ic.minLayer[y] - layerAdvance[y]
			}
		}
	}
	for i := 0; i < len(ic.subCircuitInsnIds); i++ {
		if ic.isUsed[nv+nhs+i] && ic.minLayer[nv+nhs+i] != ic.maxLayer[nv+nhs+i] {
			panic("unexpected situation: sub-circuit virtual variable should have equal min/max layer")
		}
	}

	for i, v := range ic.subCircuitInsnIds {
		ic.subCircuitLocMap[v] = i
	}

	// force outputLayer to be at least 1
	if ic.outputLayer < 1 {
		ic.outputLayer = 1
	}

	//fmt.Printf("[%d %d %d]\n", nv, ns, nhs)
	//fmt.Printf("%v\n", ic.isUsed)
	//fmt.Printf("%v\n", ic.minLayer)
	//fmt.Printf("%v\n", ic.maxLayer)

	for i, x := range ic.isUsed {
		if x && ic.minLayer[i] == -1 {
			panic("unexpected situation")
		}
	}

	// if (the output includes partial output of a sub circuit or the sub circuit has constraints),
	// and the sub circuit also ends at the output layer, we have to increate output layer
checkNextCircuit:
	for i, insnId := range ic.subCircuitInsnIds {
		count := 0
		for _, y := range outEdges[nv+nhs+i] {
			if ic.minLayer[y] == ic.outputLayer {
				if _, ok := ic.outputOrder[y]; ok {
					count++
				}
			} else {
				continue checkNextCircuit
			}
		}
		anyConstraint := false
		for _, v := range ctx.circuits[circuit.Instructions[insnId].SubCircuitId].combinedConstraints {
			if v != nil {
				anyConstraint = true
			}
		}
		if (count != 0 || anyConstraint) && count != len(outEdges[nv+nhs+i]) {
			ic.outputLayer++
			break
		}
	}

	// force maxLayer of output to be outputLayer
	for _, x := range circuit.Output {
		ic.maxLayer[x[0].VID0] = ic.outputLayer
	}

	// adjust minLayer of GetRandomValue variables to a larger value
	for _, insn := range circuit.Instructions {
		if insn.Type == ir.IGetRandom {
			x := insn.OutputIds[0]
			if !ic.isUsed[x] || ic.maxLayer[x] == ic.outputLayer {
				continue
			}
			ic.minLayer[x] = ic.outputLayer
			for _, y := range outEdges[x] {
				if ic.isUsed[y] && ic.minLayer[y]-layerAdvance[y] < ic.minLayer[x] {
					ic.minLayer[x] = ic.minLayer[y] - layerAdvance[y]
				}
			}
		}
	}
}
