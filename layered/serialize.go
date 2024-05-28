package layered

import (
	"math/big"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/utils"
)

// Serialize converts a RootCircuit into a byte array for storage or transmission.
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

func DeserializeRootCircuit(buf []byte) *RootCircuit {
	in := utils.NewInputBuf(buf[:len(buf)-32])
	inlast := utils.NewInputBuf(buf[len(buf)-32:])
	if in.ReadUint64() != 3626604230490605891 {
		panic("invalid file header")
	}
	rc := &RootCircuit{}
	nbCircuits := in.ReadUint64()
	rc.Circuits = make([]*Circuit, nbCircuits)
	rc.Field = inlast.ReadBigInt()
	for i := uint64(0); i < nbCircuits; i++ {
		c := &Circuit{}
		c.InputLen = in.ReadUint64()
		c.OutputLen = in.ReadUint64()
		nbSubCircuits := in.ReadUint64()
		c.SubCircuits = make([]SubCircuit, nbSubCircuits)
		for j := uint64(0); j < nbSubCircuits; j++ {
			sub := SubCircuit{}
			sub.Id = in.ReadUint64()
			nbAllocations := in.ReadUint64()
			sub.Allocations = make([]Allocation, nbAllocations)
			for k := uint64(0); k < nbAllocations; k++ {
				sub.Allocations[k].InputOffset = in.ReadUint64()
				sub.Allocations[k].OutputOffset = in.ReadUint64()
			}
			c.SubCircuits[j] = sub
		}
		nbMul := in.ReadUint64()
		c.Mul = make([]GateMul, nbMul)
		for j := uint64(0); j < nbMul; j++ {
			c.Mul[j].In0 = in.ReadUint64()
			c.Mul[j].In1 = in.ReadUint64()
			c.Mul[j].Out = in.ReadUint64()
			c.Mul[j].Coef = in.ReadBigInt()
		}
		nbAdd := in.ReadUint64()
		c.Add = make([]GateAdd, nbAdd)
		for j := uint64(0); j < nbAdd; j++ {
			c.Add[j].In = in.ReadUint64()
			c.Add[j].Out = in.ReadUint64()
			c.Add[j].Coef = in.ReadBigInt()
		}
		nbCst := in.ReadUint64()
		c.Cst = make([]GateCst, nbCst)
		for j := uint64(0); j < nbCst; j++ {
			c.Cst[j].Out = in.ReadUint64()
			c.Cst[j].Coef = in.ReadBigInt()
		}
		nbRandomCoef := in.ReadUint64()
		randomCoefIdx := make([]int, nbRandomCoef)
		for j := uint64(0); j < nbRandomCoef; j++ {
			randomCoefIdx[j] = int(in.ReadUint64())
		}
		for _, k := range randomCoefIdx {
			if k < len(c.Mul) {
				c.Mul[k].Coef = rc.Field
			} else if k < len(c.Mul)+len(c.Add) {
				c.Add[k-len(c.Mul)].Coef = rc.Field
			} else {
				c.Cst[k-len(c.Mul)-len(c.Add)].Coef = rc.Field
			}
		}
		rc.Circuits[i] = c
	}
	nbLayers := in.ReadUint64()
	rc.Layers = make([]uint64, nbLayers)
	for i := uint64(0); i < nbLayers; i++ {
		rc.Layers[i] = in.ReadUint64()
	}
	return rc
}
