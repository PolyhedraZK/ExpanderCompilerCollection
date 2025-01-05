use std::thread;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use ark_bls12_381::g2;
use circuit_std_rs::gnark::hints::register_hint;
use circuit_std_rs::logup::LogUpSingleKeyTable;
use expander_compiler::frontend::*;
use expander_config::M31ExtConfigSha2;
use num_bigint::BigInt;
use sha2::{Digest, Sha256};
use circuit_std_rs::big_int::{to_binary_hint, big_array_add};
use circuit_std_rs::sha2_m31::check_sha256;
use circuit_std_rs::gnark::emulated::field_bls12381::*;
use circuit_std_rs::gnark::emulated::field_bls12381::e2::*;
use circuit_std_rs::gnark::emulated::sw_bls12381::pairing::*;
use circuit_std_rs::gnark::emulated::sw_bls12381::g1::*;
use circuit_std_rs::gnark::emulated::sw_bls12381::g2::*;
use circuit_std_rs::gnark::element::*;
use expander_compiler::frontend::extra::*;
use circuit_std_rs::big_int::*;
use expander_compiler::{circuit::layered::InputType, frontend::*};

use crate::utils::run_circuit;


/*
type PermutationHashCircuit struct {
	Index [TableSize]frontend.Variable
	Value [TableSize]frontend.Variable
	Table [TableSize]frontend.Variable
}
*/
const TableSize: usize = 64;
declare_circuit!(PermutationHashCircuit {
    index: [Variable;TableSize],
    value: [Variable;TableSize],
    table: [Variable;TableSize],
});

impl GenericDefine<M31Config> for PermutationHashCircuit<Variable>  {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut table = LogUpSingleKeyTable::new(8);
        let mut table_key = vec![];
        for i in 0..TableSize {
            table_key.push(builder.constant(i as u32));
        }
        let mut table_values = vec![];
        for i in 0..TableSize {
            table_values.push(vec![self.table[i]]);
        }
        table.new_table(table_key, table_values);
        let mut query_values = vec![];
        for i in 0..TableSize {
            query_values.push(vec![self.value[i]]);
        }
        table.batch_query(self.index.to_vec(), query_values);
        //m31 field, repeat 3 times
        table.final_check(builder);
        table.final_check(builder);
        table.final_check(builder);
    }
}

/*
type PermutationIndicesValidatorHashesCircuit struct {
	QueryIndices            [QuerySize]frontend.Variable
	QueryValidatorHashes    [QuerySize][hash.PoseidonHashLength]frontend.Variable
	ActiveValidatorBitsHash [hash.PoseidonHashLength]frontend.Variable `gnark:",public"` //share with verifier, update every epoch
	ActiveValidatorBits     [ValidatorCount]frontend.Variable
	TableValidatorHashes    [ValidatorCount][hash.PoseidonHashLength]frontend.Variable
	RealKeys                [ValidatorCount]frontend.Variable
	ActiveKeys              [ValidatorCount]frontend.Variable
	InactiveKeys            [ValidatorCount]frontend.Variable
}
*/
const QuerySize: usize = 64;
const ValidatorCount: usize = 64;
declare_circuit!(PermutationIndicesValidatorHashesCircuit {
    query_indices: [Variable;QuerySize],
    query_validator_hashes: [[Variable;32];QuerySize],
    active_validator_bits_hash: [Variable;32],
    active_validator_bits: [Variable;ValidatorCount],
    table_validator_hashes: [[Variable;32];ValidatorCount],
    real_keys: [Variable;ValidatorCount],
    active_keys: [Variable;ValidatorCount],
    inactive_keys: [Variable;ValidatorCount],
});

impl GenericDefine<M31Config> for PermutationIndicesValidatorHashesCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        //check the activeValidatorBitsHash
        if self.active_validator_bits.len() % 16 != 0 {
            panic!("activeValidatorBits length must be multiple of 16")
        }
        let mut active_validator_16_bits = vec![];
        for i in 0..ValidatorCount/16 {
            active_validator_16_bits.push(from_binary(builder, self.active_validator_bits[i*16..(i+1)*16].to_vec()));
        }
        let active_validator_hash = hash::generic_hash(builder, active_validator_16_bits, hash::PoseidonHashLength);
        for i in 0..hash::PoseidonHashLength {
            builder.assert_is_equal(active_validator_hash[i], self.active_validator_bits_hash[i]);
        }
        //move inactive validators to the end
        let mut sorted_table_key = [Variable;ValidatorCount];
        for i in 0..ValidatorCount {
            sorted_table_key[i] = self.real_keys[i]; //if active, use curKey, else use curInactiveKey
        }
        let shift = builder.select(self.active_validator_bits[0], 0, builder.sub(0, ValidatorCount));
        builder.assert_is_equal(builder.is_zero(builder.add(sorted_table_key[0], shift)), 1); //the first key must be 0 or ValidatorCount
        for i in 1..ValidatorCount {
            //for every validator, its key can be
            //active and active: previous key + 1
            //active and inactive: previous key - ValidatorCount + 1
            //inactive and active: previous key + ValidatorCount
            //inactive and inactive: previous key
            let diff = builder.sub(sorted_table_key[i], sorted_table_key[i-1]);
            let shift = builder.select(builder.xor(self.active_validator_bits[i], self.active_validator_bits[i-1]), ValidatorCount, 0);
            let shift = builder.select(self.active_validator_bits[i], shift, builder.sub(0, shift));
            let diff = builder.add(diff, shift);
            //if current one is active, the diff must be 1. Otherwise, the diff must be 0. That is, always equal to activeValidatorBits[i]
            builder.assert_is_equal(builder.is_zero(builder.sub(diff, self.active_validator_bits[i])), 1);
        }
        //logup
        let mut logup = LogUpSingleKeyTable::new(8);
        let mut table_values = vec![];
        for i in 0..ValidatorCount {
            table_values.push(self.table_validator_hashes[i].to_vec());
        }
        //build a table with sorted key, i.e., the inactive validators have been moved to the end
        logup.new_table(sorted_table_key.to_vec(), table_values);
        //logup
        let mut query_values = vec![];
        for i in 0..QuerySize {
            query_values.push(self.query_validator_hashes[i].to_vec());
        }
        logup.batch_query(self.query_indices.to_vec(), query_values);
        logup.final_check(builder);
        logup.final_check(builder);
        logup.final_check(builder);
    }
}
/*
func (circuit *PermutationIndicesValidatorHashesCircuit) Define(api frontend.API) error {
	//check the activeValidatorBitsHash
	if len(circuit.ActiveValidatorBits)%16 != 0 {
		panic("activeValidatorBits length must be multiple of 16")
	}
	activeValidator16Bits := make([]frontend.Variable, len(circuit.ActiveValidatorBits)/16)
	for i := 0; i < len(circuit.ActiveValidatorBits); i += 16 {
		activeValidator16Bits[i/16] = api.FromBinary(circuit.ActiveValidatorBits[i : i+16]...)
	}
	activeValidatorHash := hash.GenericHash(api, activeValidator16Bits, hash.PoseidonHashLength)
	for i := 0; i < hash.PoseidonHashLength; i++ {
		api.AssertIsEqual(circuit.ActiveValidatorBitsHash[i], activeValidatorHash[i])
	}
	//move inactive validators to the end
	sortedTableKey := [ValidatorCount]frontend.Variable{}
	for i := 0; i < ValidatorCount; i++ {
		sortedTableKey[i] = circuit.RealKeys[i] //if active, use curKey, else use curInactiveKey
	}
	shift := api.Select(circuit.ActiveValidatorBits[0], 0, frontend.Variable(-ValidatorCount))
	api.AssertIsEqual(api.IsZero(api.Add(sortedTableKey[0], shift)), 1) //the first key must be 0 or ValidatorCount
	for i := 1; i < ValidatorCount; i++ {
		//for every validator, its key can be
		//active and active: previous key + 1
		//active and inactive: previous key - ValidatorCount + 1
		//inactive and active: previous key + ValidatorCount
		//inactive and inactive: previous key
		diff := api.Sub(sortedTableKey[i], sortedTableKey[i-1])
		shift := api.Select(api.Xor(circuit.ActiveValidatorBits[i], circuit.ActiveValidatorBits[i-1]), ValidatorCount, 0)
		shift = api.Select(circuit.ActiveValidatorBits[i], shift, api.Sub(0, shift))
		diff = api.Add(diff, shift)
		//if current one is active, the diff must be 1. Otherwise, the diff must be 0. That is, always equal to activeValidatorBits[i]
		api.AssertIsEqual(api.IsZero(api.Sub(diff, circuit.ActiveValidatorBits[i])), 1)
	}
	//logup
	logup.Reset()
	tableValues := make([][]frontend.Variable, len(circuit.TableValidatorHashes))
	for i := 0; i < len(circuit.TableValidatorHashes); i++ {
		tableValues[i] = make([]frontend.Variable, len(circuit.TableValidatorHashes[i]))
		copy(tableValues[i], circuit.TableValidatorHashes[i][:])
	}
	//build a table with sorted key, i.e., the inactive validators have been moved to the end
	logup.NewTable(sortedTableKey[:], tableValues)
	//logup
	queryValues := make([][]frontend.Variable, len(circuit.QueryIndices))
	for i := 0; i < len(circuit.QueryIndices); i++ {
		queryValues[i] = make([]frontend.Variable, len(circuit.QueryValidatorHashes[i]))
		copy(queryValues[i], circuit.QueryValidatorHashes[i][:])
	}
	logup.BatchQuery(circuit.QueryIndices[:], queryValues)
	logup.FinalCheck(api, logup.ColumnCombineOption)
	logup.FinalCheck(api, logup.ColumnCombineOption)
	logup.FinalCheck(api, logup.ColumnCombineOption)
	return nil
}

*/