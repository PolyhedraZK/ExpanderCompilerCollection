package layered

import "github.com/PolyhedraZK/ExpanderCompilerCollection/utils"

type Stats struct {
	// number of layers in the final circuit
	NbLayer int
	// number of circuits (or, segments)
	NbCircuit int
	// number of used input variables
	NbInput int
	// number of mul/add/cst gates in all circuits (unexpanded)
	NbTotMul    int
	NbTotAdd    int
	NbTotCst    int
	NbTotCustom map[uint64]int
	// number of mul/add/cst gates in expanded form of all layers
	NbExpandedMul    int
	NbExpandedAdd    int
	NbExpandedCst    int
	NbExpandedCustom map[uint64]int
	// number of total gates in the final circuit (except input gates)
	NbTotGates int
	// number of actually used gates used in the final circuit
	NbUsedGates int
	// total cost according to some formula
	TotalCost int
}

type circuitStats struct {
	nbSelfMul        int
	nbSelfAdd        int
	nbSelfCst        int
	nbSelfCustom     map[uint64]int
	nbExpandedMul    int
	nbExpandedAdd    int
	nbExpandedCst    int
	nbExpandedCustom map[uint64]int
}

type statsContext struct {
	rc *RootCircuit
	m  []circuitStats
}

// GetStats collects and returns statistical information about a RootCircuit,
// such as the number of layers, circuits, and different types of gates before
// and after expansion.
func (rc *RootCircuit) GetStats() Stats {
	sc := &statsContext{
		rc: rc,
		m:  make([]circuitStats, len(rc.Circuits)),
	}
	ar := Stats{
		NbTotCustom:      make(map[uint64]int),
		NbExpandedCustom: make(map[uint64]int),
	}
	for i, circuit := range rc.Circuits {
		r := &sc.m[i]
		r.nbSelfMul = len(circuit.Mul)
		r.nbSelfAdd = len(circuit.Add)
		r.nbSelfCst = len(circuit.Cst)
		r.nbSelfCustom = make(map[uint64]int)
		r.nbExpandedMul = r.nbSelfMul
		r.nbExpandedAdd = r.nbSelfAdd
		r.nbExpandedCst = r.nbSelfCst
		r.nbExpandedCustom = make(map[uint64]int)
		for _, ct := range circuit.Custom {
			r.nbSelfCustom[ct.GateType]++
			r.nbExpandedCustom[ct.GateType]++
		}
		for _, sub := range circuit.SubCircuits {
			r.nbExpandedMul += sc.m[sub.Id].nbExpandedMul * len(sub.Allocations)
			r.nbExpandedAdd += sc.m[sub.Id].nbExpandedAdd * len(sub.Allocations)
			r.nbExpandedCst += sc.m[sub.Id].nbExpandedCst * len(sub.Allocations)
			for k, v := range sc.m[sub.Id].nbExpandedCustom {
				r.nbExpandedCustom[k] += v * len(sub.Allocations)
			}
		}
		ar.NbTotMul += r.nbSelfMul
		ar.NbTotAdd += r.nbSelfAdd
		ar.NbTotCst += r.nbSelfCst
		for k, v := range r.nbSelfCustom {
			ar.NbTotCustom[k] += v
		}
	}
	for _, x := range rc.Layers {
		ar.NbExpandedMul += sc.m[x].nbExpandedMul
		ar.NbExpandedAdd += sc.m[x].nbExpandedAdd
		ar.NbExpandedCst += sc.m[x].nbExpandedCst
		for k, v := range sc.m[x].nbExpandedCustom {
			ar.NbExpandedCustom[k] += v
		}
	}
	ar.NbCircuit = len(rc.Circuits)
	ar.NbLayer = len(rc.Layers)
	inputMask, outputMask := computeMasks(rc)
	for i := 0; i < len(rc.Layers); i++ {
		ar.NbTotGates += int(rc.Circuits[rc.Layers[i]].OutputLen)
		for j := uint64(0); j < rc.Circuits[rc.Layers[i]].OutputLen; j++ {
			if outputMask[rc.Layers[i]][j] {
				ar.NbUsedGates++
			}
		}
	}
	for i := 0; i < int(rc.Circuits[rc.Layers[0]].InputLen); i++ {
		if inputMask[rc.Layers[0]][i] {
			ar.NbInput++
		}
	}
	ar.TotalCost = int(rc.Circuits[rc.Layers[0]].InputLen) * utils.CostOfInput
	ar.TotalCost += ar.NbTotGates * utils.CostOfVariable
	ar.TotalCost += ar.NbExpandedMul * utils.CostOfMulGate
	ar.TotalCost += ar.NbExpandedAdd * utils.CostOfAddGate
	ar.TotalCost += ar.NbExpandedCst * utils.CostOfCstGate
	return ar
}
