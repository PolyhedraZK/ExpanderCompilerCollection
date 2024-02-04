package ir

type Stats struct {
	// number of input gates in root circuit
	NbRootInput int
	// number of terms in all expressions, including instructions, outputs, and constraints
	NbTotTerms int
	// number of terms if all circuits are expanded
	NbExpandedTerms int
	// number of constraints in expanded circuit
	NbConstraints int
}

type circuitStats struct {
	nbHintInput     int
	nbSelfTerms     int
	nbExpandedTerms int
	nbConstraints   int
}

type statsContext struct {
	rc *RootCircuit
	m  map[uint64]*circuitStats
}

func (rc *RootCircuit) GetStats() Stats {
	sc := &statsContext{
		rc: rc,
		m:  make(map[uint64]*circuitStats),
	}
	r := Stats{}
	for id := range rc.Circuits {
		sc.calcCircuitStats(id)
		r.NbTotTerms += sc.m[id].nbSelfTerms
	}
	r.NbRootInput = rc.Circuits[0].NbExternalInput + sc.m[0].nbHintInput
	r.NbExpandedTerms = sc.m[0].nbExpandedTerms
	r.NbConstraints = sc.m[0].nbConstraints
	return r
}

func (sc *statsContext) calcCircuitStats(id uint64) {
	if _, ok := sc.m[id]; ok {
		return
	}
	circuit := sc.rc.Circuits[id]
	r := &circuitStats{}
	for _, insn := range circuit.Instructions {
		for _, in := range insn.Inputs {
			r.nbSelfTerms += len(in)
		}
		if insn.Type == IHint {
			r.nbHintInput += len(insn.OutputIds)
		} else if insn.Type == ISubCircuit {
			sc.calcCircuitStats(insn.SubCircuitId)
			r.nbExpandedTerms += sc.m[insn.SubCircuitId].nbExpandedTerms
			r.nbConstraints += sc.m[insn.SubCircuitId].nbConstraints
			r.nbHintInput += sc.m[insn.SubCircuitId].nbHintInput
		}
	}
	for _, expr := range circuit.Output {
		r.nbSelfTerms += len(expr)
	}
	for _, expr := range circuit.Constraints {
		r.nbSelfTerms += len(expr)
	}
	r.nbConstraints += len(circuit.Constraints)
	r.nbExpandedTerms += r.nbSelfTerms
	sc.m[id] = r
}
