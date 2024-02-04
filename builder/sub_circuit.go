package builder

import (
	"github.com/consensys/gnark/frontend"
)

// the unique identifier to a sub-circuit function, including
// 1. function name
// 2. non frontend.Variable function args
// 3. dimension of frontend.Variable function args
//type CircuitId uint64

// TODO: support various args by reflect
type SubCircuitFunc func(api frontend.API, input []frontend.Variable) []frontend.Variable

type CircuitRegistry map[uint64]*builder

func (builder *builder) MemorizedCall(f SubCircuitFunc, input []frontend.Variable) []frontend.Variable {
	/*h := sha256.Sum256([]byte(fmt.Sprintf("%p_%d", f, len(input))))
	circuitId := CircuitId(binary.LittleEndian.Uint64(h[:8]))

	if _, ok := builder.root.registry[circuitId]; !ok {
		ib := builder.root.newBuilder(len(input))
		builder.root.registry[circuitId] = ib
		output := f(ib, input)
	}*/

	panic("TODO")
	return nil
}

func MemorizedFunc(f SubCircuitFunc) SubCircuitFunc {
	return func(api frontend.API, input []frontend.Variable) []frontend.Variable {
		return api.(*builder).MemorizedCall(f, input)
	}
}
