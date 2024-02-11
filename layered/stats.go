package layered

type Stats struct {
	// number of layers in the final circuit
	NbLayer int
	// number of circuits
	NbCircuit int
	// number of mul/add/cst gates in all circuits (unexpanded)
	NbTotMul int
	NbTotAdd int
	NbTotCst int
	// number of mul/add/cst gates in expanded form of all layers
	NbExpandedMul int
	NbExpandedAdd int
	NbExpandedCst int
	// number of total gates in the final circuit (except input gates)
	NbTotGates int
	// number of actually used gates used in the final circuit
	NbUsedGates int
}

type circuitStats struct {
	nbSelfMul     int
	nbSelfAdd     int
	nbSelfCst     int
	nbExpandedMul int
	nbExpandedAdd int
	nbExpandedCst int
}

type statsContext struct {
	rc *RootCircuit
	m  []circuitStats
}

func (rc *RootCircuit) GetStats() Stats {
	sc := &statsContext{
		rc: rc,
		m:  make([]circuitStats, len(rc.Circuits)),
	}
	ar := Stats{}
	for i, circuit := range rc.Circuits {
		r := &sc.m[i]
		r.nbSelfMul = len(circuit.Mul)
		r.nbSelfAdd = len(circuit.Add)
		r.nbSelfCst = len(circuit.Cst)
		r.nbExpandedMul = r.nbSelfMul
		r.nbExpandedAdd = r.nbSelfAdd
		r.nbExpandedCst = r.nbSelfCst
		for _, sub := range circuit.SubCircuits {
			r.nbExpandedMul += sc.m[sub.Id].nbExpandedMul * len(sub.Allocations)
			r.nbExpandedAdd += sc.m[sub.Id].nbExpandedAdd * len(sub.Allocations)
			r.nbExpandedCst += sc.m[sub.Id].nbExpandedCst * len(sub.Allocations)
		}
		ar.NbTotMul += r.nbSelfMul
		ar.NbTotAdd += r.nbSelfAdd
		ar.NbTotCst += r.nbSelfCst
	}
	for _, x := range rc.Layers {
		ar.NbExpandedMul += sc.m[x].nbExpandedMul
		ar.NbExpandedAdd += sc.m[x].nbExpandedAdd
		ar.NbExpandedCst += sc.m[x].nbExpandedCst
	}
	ar.NbCircuit = len(rc.Circuits)
	ar.NbLayer = len(rc.Layers)
	_, outputMask := computeMasks(rc)
	for i := 1; i < len(rc.Layers); i++ {
		ar.NbTotGates += int(rc.Circuits[rc.Layers[i]].OutputLen)
		for j := uint64(0); j < rc.Circuits[rc.Layers[i]].OutputLen; j++ {
			if outputMask[rc.Layers[i]][j] {
				ar.NbUsedGates++
			}
		}
	}
	return ar
}
