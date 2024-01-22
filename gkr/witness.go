package gkr

import (
	"encoding/binary"
	"fmt"
	"math/big"

	"github.com/Zklib/gkr-compiler/gkr/expr"
	fr_bn254 "github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
)

type witness []*big.Int

func (builder *builder) getWitness(assignment frontend.Circuit) witness {
	wit, err := frontend.NewWitness(assignment, builder.Field())
	if err != nil {
		panic(err)
	}
	vec := wit.Vector().(fr_bn254.Vector)

	nInt, nSec, nPub := builder.cs.GetNbVariables()
	values := make([]constraint.Element, nInt+nSec+nPub)
	filled := make([]bool, nInt+nSec+nPub)
	values[0] = builder.tOne

	calcExpr := func(e expr.Expression) constraint.Element {
		res := constraint.Element{}
		for _, term := range e {
			if !filled[term.VID0] || !filled[term.VID1] {
				panic("unexpected: unfilled values")
			}
			x := builder.cs.Mul(values[term.VID0], values[term.VID1])
			x = builder.cs.Mul(x, term.Coeff)
			res = builder.cs.Add(res, x)
		}
		return res
	}

	for i, x := range vec {
		var t big.Int
		x.BigInt(&t)
		values[i+1] = builder.cs.FromInterface(t)
	}

	if len(vec)+1 != nSec+nPub {
		panic("unexpected: variable count mismatch")
	}
	for i := 0; i < nSec+nPub; i++ {
		filled[i] = true
	}

	for _, hint := range builder.hints {
		in := make([]*big.Int, len(hint.inputs))
		out := make([]*big.Int, len(hint.outputIds))

		for i, e := range hint.inputs {
			in[i] = builder.cs.ToBigInt(calcExpr(e))
		}
		for i := 0; i < len(hint.outputIds); i++ {
			out[i] = big.NewInt(0)
		}

		if hint.f == nil {
			out[0].Set(in[0])
		} else {
			err := hint.f(builder.Field(), in, out)
			if err != nil {
				panic(err)
			}
		}

		for i, x := range hint.outputIds {
			if filled[x] {
				panic("unexpected: filled twice")
			}
			filled[x] = true
			values[x] = builder.cs.FromInterface(out[i])
		}
	}

	/*for i := 0; i < len(values); i++ {
		fmt.Printf("v%d=%s\n", i, builder.cs.ToBigInt(values[i]).String())
	}*/

	if !values[builder.output].IsZero() {
		panic("witness doesn't safisfy the requirements")
	}

	var res witness
	if builder.circuit.pad2n {
		n := nextPowerOfTwo(len(builder.inputVariableIdx), true)
		res = make(witness, n)
		for i := len(builder.inputVariableIdx); i < n; i++ {
			res[i] = big.NewInt(0)
		}
	} else {
		res = make(witness, len(builder.inputVariableIdx))
	}
	for i, x := range builder.inputVariableIdx {
		res[i] = builder.cs.ToBigInt(values[x])
	}

	return res
}

type outputBuf struct {
	buf []byte
}

func (o *outputBuf) appendBigInt(x *big.Int) {
	zbuf := make([]byte, 32)
	b := x.Bytes()
	for i := 0; i < len(b); i++ {
		zbuf[i] = b[len(b)-i-1]
	}
	for i := len(b); i < 32; i++ {
		zbuf[i] = 0
	}
	o.buf = append(o.buf, zbuf...)
}

func (o *outputBuf) appendUint32(x uint32) {
	o.buf = binary.LittleEndian.AppendUint32(o.buf, x)
}

func (o *outputBuf) appendUint64(x uint64) {
	o.buf = binary.LittleEndian.AppendUint64(o.buf, x)
}

func (w *witness) Serialize() []byte {
	buf := outputBuf{}
	buf.appendUint32(1)
	for _, x := range *w {
		buf.appendBigInt(x)
	}
	return buf.buf
}

func (w *witness) Print() {
	fmt.Println("==============================")
	for _, x := range *w {
		fmt.Println(x.String())
	}
}
