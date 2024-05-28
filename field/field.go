package field

import (
	"fmt"
	"math/big"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/field/bn254"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/field/m31"
	"github.com/consensys/gnark/constraint"
)

type Field interface {
	constraint.Field
	Field() *big.Int
	FieldBitLen() int
}

func GetFieldFromOrder(x *big.Int) Field {
	if x.Cmp(bn254.ScalarField) == 0 {
		return &bn254.Field{}
	}
	if x.Cmp(m31.ScalarField) == 0 {
		return &m31.Field{}
	}
	panic(fmt.Sprintf("unknown field %v", x))
}
