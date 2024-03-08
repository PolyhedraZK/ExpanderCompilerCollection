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

type InputBuf struct {
	buf []byte
}

func NewInputBuf(buf []byte) *InputBuf {
	return &InputBuf{buf: buf}
}

func (i *InputBuf) ReadUint32() uint32 {
	x := binary.LittleEndian.Uint32(i.buf[:4])
	i.buf = i.buf[4:]
	return x
}

func (i *InputBuf) ReadUint64() uint64 {
	x := binary.LittleEndian.Uint64(i.buf[:8])
	i.buf = i.buf[8:]
	return x
}

func (i *InputBuf) ReadBigInt() *big.Int {
	zbuf := make([]byte, 32)
	for j := 0; j < 32; j++ {
		zbuf[j] = i.buf[31-j]
	}
	x := new(big.Int).SetBytes(zbuf)
	i.buf = i.buf[32:]
	return x
}
