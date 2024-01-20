package gkr

import (
	"fmt"
	"math/big"

	"github.com/Zklib/gkr-compiler/gkr/expr"
	"github.com/consensys/gnark/frontend"
)

type gateType uint32

const (
	gateDummy  gateType = 2
	gateInput  gateType = 3
	gateRelay  gateType = 10
	gateHybrid gateType = 14
)

type gate struct {
	gateType  gateType
	gateParam []uint64
	op        []uint64
	coef      []*big.Int
}

type layer struct {
	gates []gate
}

type circuit struct {
	layers []layer
	pad2n  bool
}

// finalize will convert conditions to a single output wire
func (builder *builder) finalize() {
	res := make([]frontend.Variable, len(builder.constraints))
	for i, e := range builder.constraints {
		res[i] = e
	}
	var out frontend.Variable
	if len(res) == 0 {
		out = builder.eOne
	} else if len(res) == 1 {
		out = res[0]
	} else {
		out = builder.Mul(res[0], res[1], res[2:]...)
	}
	finalOut := builder.asInternalVariable(out.(expr.Expression))
	builder.output = finalOut[0].VID0
	builder.constraints = nil
}

// TODO: optimize this
func (builder *builder) compile(pad2n bool) {
	nInt, nSec, nPub := builder.cs.GetNbVariables()
	n := nInt + nSec + nPub
	minLayer := make([]int, n) // the first layer it can be computed
	maxLayer := make([]int, n) // the last layer it will be used
	inDeg := make([]int, n)
	inEdges := make([][]int, n)
	outEdges := make([][]int, n)
	varExpr := make(map[int]expr.Expression)
	for i := 0; i < n; i++ {
		minLayer[i] = -1
	}
	for i := 0; i < nSec+nPub; i++ {
		minLayer[i] = 0
	}

	// get all input wires and build the graph
	for _, hint := range builder.hints {
		if hint.f == nil {
			e := hint.inputs[0]
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
			x := hint.outputIds[0]
			varExpr[x] = e
			inDeg[x] = len(usedVar)
			inEdges[x] = make([]int, 0, len(usedVar))
			for y := range usedVar {
				inEdges[x] = append(inEdges[x], y)
				outEdges[y] = append(outEdges[y], x)
			}
		} else {
			for _, x := range hint.outputIds {
				minLayer[x] = 0
			}
		}
	}

	// bfs from output wire
	isUsed := make([]bool, n)
	q := make([]int, 1, n)
	q[0] = builder.output
	isUsed[q[0]] = true
	for i := 0; i < len(q); i++ {
		x := q[i]
		for _, y := range inEdges[x] {
			if !isUsed[y] {
				isUsed[y] = true
				q = append(q, y)
			}
		}
	}

	// toposort and compute the layer (depth) of each variable
	q = q[:0]
	for i := 0; i < n; i++ {
		if minLayer[i] == 0 && isUsed[i] {
			q = append(q, i)
		}
	}
	for i := 0; i < len(q); i++ {
		x := q[i]
		for _, y := range outEdges[x] {
			if isUsed[y] {
				if minLayer[y] < minLayer[x]+1 {
					minLayer[y] = minLayer[x] + 1
				}
				inDeg[y]--
				if inDeg[y] == 0 {
					q = append(q, y)
				}
			}
		}
	}

	// compute maxLayer
	for i := 0; i < n; i++ {
		maxLayer[i] = minLayer[i]
	}
	for i := 0; i < n; i++ {
		for _, y := range outEdges[i] {
			if isUsed[y] && minLayer[y]-1 > maxLayer[i] {
				maxLayer[i] = minLayer[y] - 1
			}
		}
	}

	// initialize the variables idx in layers
	nLayers := minLayer[builder.output] + 1
	layerVarIdx := make([][]int, nLayers)
	layerVarLoc := make([]map[int]int, nLayers)
	for i := 0; i < nLayers; i++ {
		layerVarLoc[i] = make(map[int]int)
	}
	for i := 0; i < n; i++ {
		if !isUsed[i] {
			continue
		}
		for j := minLayer[i]; j <= maxLayer[i]; j++ {
			k := len(layerVarIdx[j])
			layerVarLoc[j][i] = k
			layerVarIdx[j] = append(layerVarIdx[j], i)
		}
	}

	// build the circuit
	circuit := circuit{
		layers: make([]layer, nLayers),
	}
	for l := 0; l < nLayers; l++ {
		gates := make([]gate, len(layerVarIdx[l]))
		for layerId, globalId := range layerVarIdx[l] {
			gate := &gates[layerId]
			// if l==0, it's input
			// if this is the first layer for the variable, it's hybrid
			// otherwise it's relay
			if l == 0 {
				gate.gateType = gateInput
			} else if l == minLayer[globalId] {
				gate.gateType = gateHybrid
				var0 := []expr.Term{}
				var1 := []expr.Term{}
				var2 := []expr.Term{}
				for _, term := range varExpr[globalId] {
					if term.Coeff.IsZero() {
						continue
					}
					if term.VID0 == 0 {
						var0 = append(var0, term)
					} else if term.VID1 == 0 {
						term.VID0 = layerVarLoc[l-1][term.VID0]
						var1 = append(var1, term)
					} else {
						term.VID0 = layerVarLoc[l-1][term.VID0]
						term.VID1 = layerVarLoc[l-1][term.VID1]
						var2 = append(var2, term)
					}
				}
				for _, term := range var2 {
					gate.op = append(gate.op, uint64(term.VID0), uint64(term.VID1))
					gate.coef = append(gate.coef, builder.cs.ToBigInt(term.Coeff))
				}
				for _, term := range var1 {
					gate.op = append(gate.op, uint64(term.VID0))
					gate.coef = append(gate.coef, builder.cs.ToBigInt(term.Coeff))
				}
				for _, term := range var0 {
					gate.coef = append(gate.coef, builder.cs.ToBigInt(term.Coeff))
				}
				gate.gateParam = []uint64{uint64(len(var2)), uint64(len(var1)), uint64(len(var0))}
			} else {
				gate.gateType = gateRelay
				gate.op = append(gates[layerId].op, uint64(layerVarLoc[l-1][globalId]))
			}
		}
		circuit.layers[l] = layer{
			gates: gates,
		}
	}

	// finally, set the results
	circuit.pad2n = pad2n
	builder.circuit = circuit
	builder.inputVariableIdx = layerVarIdx[0]
}

func nextPowerOfTwo(x int, is4 bool) int {
	// compute pad to 2^n gates (and 4^n for first layer)
	// and n>=1
	padk := 1
	for x > (1 << padk) {
		padk++
	}
	if is4 && padk%2 != 0 {
		padk++
	}
	return 1 << padk
}

func (c *circuit) Serialize() []byte {
	buf := outputBuf{}

	buf.appendUint32(uint32(len(c.layers)))
	if c.pad2n {
		buf.appendUint64(uint64(nextPowerOfTwo(len(c.layers[0].gates), true)))
	} else {
		buf.appendUint64(uint64(len(c.layers[0].gates)))
	}
	for i := 1; i < len(c.layers); i++ {
		gates := c.layers[i].gates
		n := nextPowerOfTwo(len(gates), false)
		if !c.pad2n {
			n = len(gates)
		}
		if i+1 == len(c.layers) {
			n = 1
		}
		buf.appendUint64(uint64(n))
		for _, gate := range gates {
			buf.appendUint32(uint32(gate.gateType))
			if gate.gateType == gateRelay {
				buf.appendUint64(gate.op[0])
			} else if gate.gateType == gateHybrid {
				var2OpSize := gate.gateParam[0]
				var1OpSize := gate.gateParam[1]
				var0OpSize := gate.gateParam[2]
				buf.appendUint32(uint32(var2OpSize))
				for i := 0; i < int(var2OpSize); i++ {
					buf.appendUint64(gate.op[i*2])
					buf.appendUint64(gate.op[i*2+1])
				}
				for i := 0; i < int(var2OpSize); i++ {
					buf.appendBigInt(gate.coef[i])
				}
				buf.appendUint32(uint32(var1OpSize))
				for i := int(var2OpSize * 2); i < int(var2OpSize*2+var1OpSize); i++ {
					buf.appendUint64(gate.op[i])
				}
				for i := int(var2OpSize); i < int(var2OpSize+var1OpSize); i++ {
					buf.appendBigInt(gate.coef[i])
				}
				buf.appendUint32(uint32(var0OpSize))
				for i := int(var2OpSize + var1OpSize); i < int(var2OpSize+var1OpSize+var0OpSize); i++ {
					buf.appendBigInt(gate.coef[i])
				}
			}
		}
		for i := 0; i < n-len(gates); i++ {
			buf.appendUint32(uint32(gateDummy))
		}
	}
	return buf.buf
}

func (c *circuit) Print() {
	for i := 0; i < len(c.layers); i++ {
		gates := c.layers[i].gates
		fmt.Println("==============================")
		fmt.Printf("layer %d: %d gates\n", i, len(gates))
		for i, gate := range gates {
			if gate.gateType == gateInput {
				fmt.Printf("    gate %d: input\n", i)
			} else if gate.gateType == gateRelay {
				fmt.Printf("    gate %d: relay %d\n", i, gate.op[0])
			} else if gate.gateType == gateHybrid {
				fmt.Printf("    gate %d: hybrid\n", i)
				var2OpSize := gate.gateParam[0]
				var1OpSize := gate.gateParam[1]
				var0OpSize := gate.gateParam[2]
				for i := 0; i < int(var2OpSize); i++ {
					fmt.Printf("        v%d*v%d*%s\n", gate.op[i*2], gate.op[i*2+1], gate.coef[i].String())
				}
				for i := int(var2OpSize); i < int(var2OpSize+var1OpSize); i++ {
					fmt.Printf("        v%d*%s\n", gate.op[i+int(var2OpSize)], gate.coef[i].String())
				}
				for i := int(var2OpSize + var1OpSize); i < int(var2OpSize+var1OpSize+var0OpSize); i++ {
					fmt.Printf("        %s\n", gate.coef[i].String())
				}
			}
		}
	}
}
