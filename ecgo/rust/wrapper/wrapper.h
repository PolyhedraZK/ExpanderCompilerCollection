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

typedef PointerResult (*load_field_array_func)(ByteArray data, uint64_t config_id);
PointerResult load_field_array(void *f, ByteArray data, uint64_t config_id) {
    return ((load_field_array_func) f)(data, config_id);
}

typedef PointerResult (*dump_field_array_func)(void* field_array, ByteArray buf, uint64_t config_id);
PointerResult dump_field_array(void *f, void* field_array, ByteArray buf, uint64_t config_id) {
    return ((dump_field_array_func) f)(field_array, buf, config_id);
}

typedef PointerResult (*load_witness_solver_func)(ByteArray data, uint64_t config_id);
PointerResult load_witness_solver(void *f, ByteArray data, uint64_t config_id) {
    return ((load_witness_solver_func) f)(data, config_id);
}

typedef PointerResult (*dump_witness_solver_func)(void* witness_solver, ByteArray buf, uint64_t config_id);
PointerResult dump_witness_solver(void *f, void* witness_solver, ByteArray buf, uint64_t config_id) {
    return ((dump_witness_solver_func) f)(witness_solver, buf, config_id);
}

typedef PointerResult (*solve_witnesses_func)(void* witness_solver, void* raw_inputs_vec, uint64_t num_witnesses, void* hint_caller, uint64_t config_id);
PointerResult solve_witnesses(void *f, void* witness_solver, void* raw_inputs_vec, uint64_t num_witnesses, void* hint_caller, uint64_t config_id) {
    return ((solve_witnesses_func) f)(witness_solver, raw_inputs_vec, num_witnesses, hint_caller, config_id);
}

extern char* hintCallBack(uint64_t hint_id, uint8_t* inputs, uint64_t inputs_len, uint8_t* outputs, uint64_t outputs_len, uint64_t config_id);

typedef uint64_t (*abi_version_func)();
uint64_t abi_version(void *f) {
    return ((abi_version_func) f)();
}