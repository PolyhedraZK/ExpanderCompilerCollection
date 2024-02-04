package layered

import (
	"fmt"
	"math/big"

	"github.com/Zklib/gkr-compiler/circuitir"
	"github.com/Zklib/gkr-compiler/utils"
	"github.com/consensys/gnark/constraint"
)

type Witness []*big.Int

func GetWitness(rc *circuitir.RootCircuit, idx []int, values []constraint.Element, circuit *Circuit) Witness {
	output := rc.Circuits[0].Output[0][0].VID0
	if !values[output].IsZero() {
		panic("witness doesn't safisfy the requirements")
	}

	var res Witness
	if circuit.pad2n {
		n := nextPowerOfTwo(len(idx), true)
		res = make(Witness, n)
		for i := len(idx); i < n; i++ {
			res[i] = big.NewInt(0)
		}
	} else {
		res = make(Witness, len(idx))
	}
	for i, x := range idx {
		res[i] = rc.Field.ToBigInt(values[x])
	}
	return res
}

func (w *Witness) Serialize() []byte {
	buf := utils.OutputBuf{}
	buf.AppendUint32(1)
	for _, x := range *w {
		buf.AppendBigInt(x)
	}
	return buf.Bytes()
}

func (w *Witness) Print() {
	fmt.Println("==============================")
	for _, x := range *w {
		fmt.Println(x.String())
	}
}
