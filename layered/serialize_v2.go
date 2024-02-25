package layered

import (
	"math/big"

	"github.com/Zklib/gkr-compiler/utils"
)

func (rc *RootCircuit) Serialize() []byte {
	o := utils.OutputBuf{}
	zero := big.NewInt(0)
	o.AppendUint64(3626604230490605891)
	o.AppendUint64(uint64(len(rc.Circuits)))
	for _, c := range rc.Circuits {
		o.AppendUint64(c.InputLen)
		o.AppendUint64(c.OutputLen)
		o.AppendUint64(uint64(len(c.SubCircuits)))
		for _, sub := range c.SubCircuits {
			o.AppendUint64(sub.Id)
			o.AppendUint64(uint64(len(sub.Allocations)))
			for _, a := range sub.Allocations {
				o.AppendUint64(a.InputOffset)
				o.AppendUint64(a.OutputOffset)
			}
		}
		randomCoefIdx := []int{}
		o.AppendUint64(uint64(len(c.Mul)))
		for i, m := range c.Mul {
			o.AppendUint64(m.In0)
			o.AppendUint64(m.In1)
			o.AppendUint64(m.Out)
			if m.Coef.Cmp(rc.Field) == 0 {
				randomCoefIdx = append(randomCoefIdx, i)
				o.AppendBigInt(zero)
			} else {
				o.AppendBigInt(m.Coef)
			}
		}
		o.AppendUint64(uint64(len(c.Add)))
		for i, a := range c.Add {
			o.AppendUint64(a.In)
			o.AppendUint64(a.Out)
			if a.Coef.Cmp(rc.Field) == 0 {
				randomCoefIdx = append(randomCoefIdx, i+len(c.Mul))
				o.AppendBigInt(zero)
			} else {
				o.AppendBigInt(a.Coef)
			}
		}
		o.AppendUint64(uint64(len(c.Cst)))
		for i, cst := range c.Cst {
			o.AppendUint64(cst.Out)
			if cst.Coef.Cmp(rc.Field) == 0 {
				randomCoefIdx = append(randomCoefIdx, i+len(c.Mul)+len(c.Add))
				o.AppendBigInt(zero)
			} else {
				o.AppendBigInt(cst.Coef)
			}
		}
		o.AppendUint64(uint64(len(randomCoefIdx)))
		for _, idx := range randomCoefIdx {
			o.AppendUint64(uint64(idx))
		}
	}
	o.AppendUint64(uint64(len(rc.Layers)))
	for _, l := range rc.Layers {
		o.AppendUint64(l)
	}
	o.AppendBigInt(rc.Field)
	return o.Bytes()
}
