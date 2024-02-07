package ir

type InputSolver []InputSolverInstruction

type InputSolverInstruction struct {
	// instruction id
	// specially, for the global input, InsnId == 1 << 62
	InsnId int
	// if this is a hint instruction, InputIds[i] == j -> insn.OutputIds[i] should be put to j-th global input
	CircuitInputIds []int
	// if this is a sub circuit instruction, solve it recursively
	SubCircuit InputSolver
}
