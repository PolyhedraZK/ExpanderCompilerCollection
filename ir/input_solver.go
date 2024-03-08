package ir

import (
	"math/big"

	"github.com/Zklib/gkr-compiler/expr"
	fr_bn254 "github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
)

// group is the minimum schedule unit, and this is the rough number of field operations (multiplications) per group
const NbFieldOperationsPerGroup = 1024

type InputOrder struct {
	Insn            []InputOrderInstruction
	CircuitInputIds []int
	InputLen        int
}

type InputOrderInstruction struct {
	// if this is a hint instruction, InputIds[i] == j -> insn.OutputIds[i] should be put to j-th global input
	CircuitInputIds []int
	// if this is a sub circuit instruction, solve it recursively
	SubCircuit []InputOrderInstruction
}

type InputSolver struct {
	RootCircuit       *RootCircuit
	InputOrder        *InputOrder
	CircuitsSolveInfo map[uint64]*CircuitSolveInfo
}

type CircuitSolveInfo struct {
	// the dimensions are: which layer -> eval group -> instructions
	SolveOrder [][][]int
	NbVars     int
	// number of necessary done signals for each layer ( = num group + num sub_circuit)
	LayerDoneSignals []int
	MaxChanSize      int
}

// GetInputSolver calculates the input solver for the given root circuit and input order
// It does a toposort on the instructions
func GetInputSolver(rc *RootCircuit, od *InputOrder) *InputSolver {
	res := &InputSolver{
		RootCircuit:       rc,
		InputOrder:        od,
		CircuitsSolveInfo: make(map[uint64]*CircuitSolveInfo),
	}
	for id, c := range rc.Circuits {
		si := &CircuitSolveInfo{}
		res.CircuitsSolveInfo[id] = si

		n := c.NbExternalInput
		for _, insn := range c.Instructions {
			for _, x := range insn.OutputIds {
				if x > n {
					n = x
				}
			}
		}
		n++
		si.NbVars = n

		outEdges := make([][]int, n)
		varInsnId := make([]int, n)
		for i := 0; i <= c.NbExternalInput; i++ {
			varInsnId[i] = -1
		}
		remDeg := make([]int, len(c.Instructions))
		for i, insn := range c.Instructions {
			for _, x := range insn.OutputIds {
				varInsnId[x] = i
			}
			inEdges := make(map[int]bool)
			for _, x := range insn.Inputs {
				for _, t := range x {
					if varInsnId[t.VID0] != -1 {
						inEdges[varInsnId[t.VID0]] = true
					}
					if varInsnId[t.VID1] != -1 {
						inEdges[varInsnId[t.VID1]] = true
					}
				}
			}
			for x := range inEdges {
				outEdges[x] = append(outEdges[x], i)
			}
			remDeg[i] = len(inEdges)
		}
		q := make([]int, 0, len(c.Instructions))
		for i, x := range remDeg {
			if x == 0 {
				q = append(q, i)
			}
		}
		for qi := 0; qi < len(c.Instructions); qi++ {
			i := q[qi]
			for _, j := range outEdges[i] {
				remDeg[j]--
				if remDeg[j] == 0 {
					q = append(q, j)
				}
			}
		}
		dep := make([]int, len(c.Instructions))
		maxDep := 0
		for _, i := range q {
			if dep[i] > maxDep {
				maxDep = dep[i]
			}
			for _, j := range outEdges[i] {
				if dep[j] < dep[i]+1 {
					dep[j] = dep[i] + 1
				}
			}
		}
		soTmp := make([][]int, maxDep+1)
		for i := 0; i < len(c.Instructions); i++ {
			soTmp[dep[i]] = append(soTmp[dep[i]], i)
		}
		si.SolveOrder = make([][][]int, maxDep+1)
		si.LayerDoneSignals = make([]int, maxDep+1)
		for i := 0; i <= maxDep; i++ {
			si.SolveOrder[i] = make([][]int, 0)
			lastGroup := []int{}
			lastGroupCost := 0
			for _, j := range soTmp[i] {
				if c.Instructions[j].Type == ISubCircuit {
					si.LayerDoneSignals[i]++
				}
				cost := 0
				for _, e := range c.Instructions[j].Inputs {
					cost += len(e)
				}
				if len(lastGroup)+cost > NbFieldOperationsPerGroup {
					si.SolveOrder[i] = append(si.SolveOrder[i], lastGroup)
					lastGroup = []int{}
					lastGroupCost = 0
				}
				lastGroup = append(lastGroup, j)
				lastGroupCost += cost
			}
			if len(lastGroup) > 0 {
				si.SolveOrder[i] = append(si.SolveOrder[i], lastGroup)
			}
			si.LayerDoneSignals[i] += len(si.SolveOrder[i])
			if si.LayerDoneSignals[i] > si.MaxChanSize {
				si.MaxChanSize = si.LayerDoneSignals[i]
			}
		}
	}
	return res
}

type inputSolveCtx struct {
	solver      *InputSolver
	globalInput []*big.Int
	taskQueue   chan inputSolveTask
	err         chan error
}

type circuitSolveCtx struct {
	circuit   *Circuit
	si        *CircuitSolveInfo
	values    []constraint.Element
	inputInsn []InputOrderInstruction
}

type inputSolveTask struct {
	csc      *circuitSolveCtx
	insns    []int
	output   []constraint.Element
	callback chan bool
}

func (solver *InputSolver) SolveInput(assignment frontend.Circuit, nbThreads int) ([]*big.Int, error) {
	rc := solver.RootCircuit
	od := solver.InputOrder
	wit, err := frontend.NewWitness(assignment, rc.Field.Field())
	if err != nil {
		panic(err)
	}
	vec := wit.Vector().(fr_bn254.Vector)

	ctx := &inputSolveCtx{
		solver:      solver,
		globalInput: make([]*big.Int, od.InputLen),
		taskQueue:   make(chan inputSolveTask, 1024),
		err:         make(chan error, 1),
	}
	input := make([]constraint.Element, len(vec))
	for i, x := range vec {
		var t big.Int
		x.BigInt(&t)
		input[i] = rc.Field.FromInterface(t)
		p := od.CircuitInputIds[i]
		if p != -1 {
			ctx.globalInput[p] = &t
		}
	}

	for i := 0; i < nbThreads; i++ {
		go ctx.worker()
	}

	output := make([]constraint.Element, len(rc.Circuits[0].Output))
	callback := make(chan bool, 1)
	go ctx.solve(0, input, od.Insn, output, callback)
	<-callback
	select {
	case err := <-ctx.err:
		return nil, err
	default:
	}

	for i, x := range ctx.globalInput {
		if x == nil {
			ctx.globalInput[i] = big.NewInt(0)
		}
	}

	return ctx.globalInput, nil
}

func calcExpr(e expr.Expression, values []constraint.Element, field constraint.R1CS) constraint.Element {
	res := constraint.Element{}
	for _, term := range e {
		x := field.Mul(values[term.VID0], values[term.VID1])
		x = field.Mul(x, term.Coeff)
		res = field.Add(res, x)
	}
	return res
}

func (isc *inputSolveCtx) worker() {
	field := isc.solver.RootCircuit.Field
	var gin []constraint.Element
	for task := range isc.taskQueue {
		csc := task.csc
		inputInsn := csc.inputInsn
		if len(task.output) != 0 {
			for i, e := range csc.circuit.Output {
				task.output[i] = calcExpr(e, csc.values, field)
			}
		} else {
			for _, insnId := range task.insns {
				insn := csc.circuit.Instructions[insnId]
				var in []constraint.Element
				outOffset := insn.OutputIds[0]
				if insn.Type == ISubCircuit {
					in = make([]constraint.Element, len(insn.Inputs))
				} else {
					if len(gin) < len(insn.Inputs) {
						gin = make([]constraint.Element, len(insn.Inputs))
					}
					in = gin[:len(insn.Inputs)]
				}
				for i, e := range insn.Inputs {
					in[i] = calcExpr(e, csc.values, field)
				}
				if insn.Type == IInternalVariable {
					csc.values[outOffset] = in[0]
				} else if insn.Type == IHint {
					inB := make([]*big.Int, len(insn.Inputs))
					outB := make([]*big.Int, len(insn.OutputIds))
					for i, e := range in {
						inB[i] = field.ToBigInt(e)
					}
					for i := 0; i < len(insn.OutputIds); i++ {
						outB[i] = big.NewInt(0)
					}
					err := insn.HintFunc(field.Field(), inB, outB)
					if err != nil {
						select {
						case isc.err <- err:
						default:
						}
					}
					for j, x := range outB {
						csc.values[j+outOffset] = field.FromInterface(x)
						//fmt.Printf("set %d %d\n", is.CircuitInputIds[i], x)
						p := inputInsn[insnId].CircuitInputIds[j]
						if p != -1 {
							isc.globalInput[p] = x
						}
					}
				} else if insn.Type == ISubCircuit {
					go isc.solve(insn.SubCircuitId, in, inputInsn[insnId].SubCircuit, csc.values[outOffset:outOffset+len(insn.OutputIds)], task.callback)
				}
			}
		}
		task.callback <- true
	}
}

func (isc *inputSolveCtx) solve(id uint64, input []constraint.Element, inputInsn []InputOrderInstruction, output []constraint.Element, callback chan bool) {
	rc := isc.solver.RootCircuit
	circuit := rc.Circuits[id]
	si := isc.solver.CircuitsSolveInfo[id]

	csc := &circuitSolveCtx{
		circuit:   circuit,
		si:        si,
		values:    make([]constraint.Element, si.NbVars),
		inputInsn: inputInsn,
	}

	csc.values[0] = rc.Field.One()

	for i, x := range input {
		csc.values[i+1] = x
	}

	subCallback := make(chan bool, si.MaxChanSize)
	for i, curLayer := range si.SolveOrder {
		for _, group := range curLayer {
			isc.taskQueue <- inputSolveTask{
				csc:      csc,
				insns:    group,
				callback: subCallback,
			}
		}
		for j := 0; j < si.LayerDoneSignals[i]; j++ {
			<-subCallback
		}
	}

	isc.taskQueue <- inputSolveTask{
		csc:      csc,
		output:   output,
		callback: callback,
	}
}
