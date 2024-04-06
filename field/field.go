package field

import (
	"fmt"
	"math/big"

	"github.com/Zklib/gkr-compiler/field/bn254"
	"github.com/Zklib/gkr-compiler/field/mersen"
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
	if x.Cmp(mersen.ScalarField) == 0 {
		return &mersen.Field{}
	}
	panic(fmt.Sprintf("unknown field %v", x))
}
