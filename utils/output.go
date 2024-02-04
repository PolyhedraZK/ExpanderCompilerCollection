package utils

import (
	"encoding/binary"
	"math/big"
)

type OutputBuf struct {
	buf []byte
}

func (o *OutputBuf) AppendBigInt(x *big.Int) {
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

func (o *OutputBuf) AppendUint32(x uint32) {
	o.buf = binary.LittleEndian.AppendUint32(o.buf, x)
}

func (o *OutputBuf) AppendUint64(x uint64) {
	o.buf = binary.LittleEndian.AppendUint64(o.buf, x)
}

func (o *OutputBuf) Bytes() []byte {
	return o.buf
}
