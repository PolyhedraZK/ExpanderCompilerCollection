package builder

import (
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/irsource"
)

// Finalize processes deferred functions, converts boolean and nonzero assertions to zero assertions,
// and adds public variables to the output.
func (r *Root) Finalize() *irsource.RootCircuit {
	if len(r.publicVariables) > 0 {
		r.finalizePublicVariables(r.publicVariables)
	}

	res := make(map[uint64]*irsource.Circuit)
	for x, b := range r.registry.m {
		res[x] = b.builder.Finalize()
	}
	return &irsource.RootCircuit{
		Circuits: res,
		Field:    r.field,
	}
}

func (builder *builder) Finalize() *irsource.Circuit {
	// defers may change during the process
	for i := 0; i < len(builder.defers); i++ {
		cb := builder.defers[i]
		err := cb(builder)
		if err != nil {
			panic(err)
		}
	}

	return &irsource.Circuit{
		Instructions: builder.instructions,
		Constraints:  builder.constraints,
		Outputs:      unwrapVariables(builder.output),
		NumInputs:    builder.nbExternalInput,
	}
}

// This function will only be called on the root circuit
func (builder *builder) finalizePublicVariables(pvIds []int) {
	for _, id := range pvIds {
		builder.output = append(builder.output, variable{id})
	}
}