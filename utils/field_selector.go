package utils

import (
	"math/big"

	"github.com/consensys/gnark/constraint"
	bn254r1cs "github.com/consensys/gnark/constraint/bn254"
)

func GetR1CSFromField(x *big.Int) constraint.R1CS {
	field := bn254r1cs.NewR1CS(0)
	if x.Cmp(field.Field()) != 0 {
		panic("currently only BN254 is supported")
	}
	return bn254r1cs.NewR1CS(0)
}
