#include <stdint.h>
#include <stdlib.h>

typedef struct {
    uint8_t* data;
    uint64_t length;
} ByteArray;

typedef struct {
    ByteArray ir_witness_gen;
    ByteArray layered;
    ByteArray error;
} CompileResult;

CompileResult compile(ByteArray ir_source, uint64_t config_id);