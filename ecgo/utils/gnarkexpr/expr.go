package gnarkexpr

import (
	"reflect"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
)

var builder frontend.Builder

type Expr interface {
	WireID() int
}

func init() {
	var err error
	builder, err = r1cs.NewBuilder(bn254.ID.ScalarField(), frontend.CompileConfig{})
	if err != nil {
		panic(err)
	}
}

// gnark uses uint32
const MaxVariables = (1 << 31) - 100

func NewVar(x int) Expr {
	if x < 0 || x >= MaxVariables {
		panic("variable id out of range")
	}
	v := builder.InternalVariable(uint32(x))
	t := reflect.ValueOf(v).Index(0).Interface().(Expr)
	if t.WireID() != x {
		panic("variable id mismatch, please check gnark version")
	}
	return t
}
