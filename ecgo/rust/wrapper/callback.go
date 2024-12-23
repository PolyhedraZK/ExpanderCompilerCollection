package wrapper

/*
#include <stdint.h>
*/
import "C"
import (
	"math/big"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/field"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/utils"
	"github.com/consensys/gnark/constraint/solver"
)

//export hintCallBack
func hintCallBack(hintId C.uint64_t, inputs *C.uint8_t, inputsLen C.uint64_t, outputs **C.uint8_t, outputsLen C.uint64_t, configId C.uint64_t) *C.char {
	err := func() error {
		field_ := field.GetFieldById(uint64(configId))
		eLen := field_.SerializedLen()
		inputsBytes := goBytes(inputs, C.uint64_t(int(inputsLen)*eLen))
		ibuf := utils.NewInputBuf(inputsBytes)
		inputsBn := make([]*big.Int, int(inputsLen))
		for i := 0; i < int(inputsLen); i++ {
			inputsBn[i] = ibuf.ReadBigInt(eLen)
		}
		outputsBn := make([]*big.Int, int(outputsLen))
		for i := 0; i < int(outputsLen); i++ {
			outputsBn[i] = big.NewInt(0)
		}
		solver.GetRegisteredHint(solver.HintID(hintId))(field_.Field(), inputsBn, outputsBn)
		obuf := utils.OutputBuf{}
		for i := 0; i < int(outputsLen); i++ {
			obuf.AppendBigInt(eLen, outputsBn[i])
		}
		obufBytes := obuf.Bytes()
		*outputs = (*C.uint8_t)(C.CBytes(obufBytes))
		return nil
	}()
	if err != nil {
		return C.CString(err.Error())
	}
	return nil
}
