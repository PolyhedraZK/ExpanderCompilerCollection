#include <stdint.h>
#include <stdlib.h>

typedef struct {
    uint8_t* data;
    uint64_t length;
} ByteArray;

typedef struct {
    void* witness_solver;
    ByteArray layered;
    ByteArray error;
} CompileResult;

typedef struct {
    void* pointer;
    ByteArray error;
} PointerResult;

typedef struct {
    ByteArray result;
    ByteArray error;
} DumpResult;

typedef struct {
    void* witness_vec;
    uint64_t num_inputs_per_witness;
    uint64_t num_public_inputs_per_witness;
    ByteArray error;
} WitnessResult;

typedef CompileResult (*compile_func)(ByteArray ir_source, uint64_t config_id);
CompileResult compile(void *f, ByteArray ir_source, uint64_t config_id) {
    return ((compile_func) f)(ir_source, config_id);
}

typedef ByteArray (*prove_circuit_file_func)(ByteArray circuit_filename, ByteArray witness, uint64_t config_id);
ByteArray prove_circuit_file(void *f, ByteArray circuit_filename, ByteArray witness, uint64_t config_id) {
    return ((prove_circuit_file_func) f)(circuit_filename, witness, config_id);
}

typedef uint8_t (*verify_circuit_file_func)(ByteArray circuit_filename, ByteArray witness, ByteArray proof, uint64_t config_id);
uint8_t verify_circuit_file(void *f, ByteArray circuit_filename, ByteArray witness, ByteArray proof, uint64_t config_id) {
    return ((verify_circuit_file_func) f)(circuit_filename, witness, proof, config_id);
}

typedef void (*free_object_func)(void* object);
void free_object(void *f, void* object) {
    ((free_object_func) f)(object);
}

typedef PointerResult (*load_field_array_func)(ByteArray data, uint64_t len, uint64_t config_id);
PointerResult load_field_array(void *f, ByteArray data, uint64_t len, uint64_t config_id) {
    return ((load_field_array_func) f)(data, len, config_id);
}

typedef PointerResult (*dump_field_array_func)(void* field_array, uint64_t* res_len, uint64_t config_id);
DumpResult dump_field_array(void *f, void* field_array, uint64_t config_id) {
    uint64_t res_len = 0;
    PointerResult res = ((dump_field_array_func) f)(field_array, &res_len, config_id);
    DumpResult result;
    result.result.data = (uint8_t*) res.pointer;
    result.result.length = res_len;
    result.error.data = res.error.data;
    result.error.length = res.error.length;
    return result;
}

typedef PointerResult (*load_witness_solver_func)(ByteArray data, uint64_t config_id);
PointerResult load_witness_solver(void *f, ByteArray data, uint64_t config_id) {
    return ((load_witness_solver_func) f)(data, config_id);
}

typedef PointerResult (*dump_witness_solver_func)(void* witness_solver, uint64_t* res_len, uint64_t config_id);
DumpResult dump_witness_solver(void *f, void* witness_solver, uint64_t config_id) {
    uint64_t res_len = 0;
    PointerResult res = ((dump_witness_solver_func) f)(witness_solver, &res_len, config_id);
    DumpResult result;
    result.result.data = (uint8_t*) res.pointer;
    result.result.length = res_len;
    result.error.data = res.error.data;
    result.error.length = res.error.length;
    return result;
}

typedef PointerResult (*solve_witnesses_func)(void* witness_solver, void* raw_inputs_vec, uint64_t num_witnesses, void* hint_caller, uint64_t config_id, uint64_t* res_num_inputs_per_witness, uint64_t* res_num_public_inputs_per_witness);
WitnessResult solve_witnesses(void *f, void* witness_solver, void* raw_inputs_vec, uint64_t num_witnesses, void* hint_caller, uint64_t config_id) {
    uint64_t num_inputs_per_witness = 0;
    uint64_t num_public_inputs_per_witness = 0;
    PointerResult res = ((solve_witnesses_func) f)(witness_solver, raw_inputs_vec, num_witnesses, hint_caller, config_id, &num_inputs_per_witness, &num_public_inputs_per_witness);
    WitnessResult result;
    result.witness_vec = res.pointer;
    result.num_inputs_per_witness = num_inputs_per_witness;
    result.num_public_inputs_per_witness = num_public_inputs_per_witness;
    result.error.data = res.error.data;
    result.error.length = res.error.length;
    return result;
}

extern char* hintCallBack(uint64_t hint_id, uint8_t* inputs, uint64_t inputs_len, uint8_t* outputs, uint64_t outputs_len, uint64_t config_id);

typedef uint64_t (*abi_version_func)();
uint64_t abi_version(void *f) {
    return ((abi_version_func) f)();
}