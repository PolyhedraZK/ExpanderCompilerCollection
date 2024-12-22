package wrapper

/*
#include <stdint.h>
*/
import "C"

//export hintCallBack
func hintCallBack(hintId C.uint64_t, inputs *C.uint8_t, inputsLen C.uint64_t, outputs *C.uint8_t, outputsLen C.uint64_t, configId C.uint64_t) *C.char {
	panic("HintCallBack is not implemented")
}
