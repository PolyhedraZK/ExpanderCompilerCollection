use std::thread;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use ark_bls12_381::g2;
use circuit_std_rs::gnark::hints::register_hint;
use circuit_std_rs::logup::LogUpRangeProofTable;
use circuit_std_rs::utils::simple_select;
use expander_compiler::frontend::*;
use expander_config::M31ExtConfigSha2;
use num_bigint::BigInt;
use sha2::{Digest, Sha256};
use circuit_std_rs::big_int::{to_binary_hint, big_array_add};
use crate::bls::check_pubkey_key_bls;
use crate::validator::ValidatorSSZ;
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
const (
	ShuffleRound           = 90
	PrepareChunkSize       = 2048
	ValidatorChunkSize     = 128 * 4
	TestValidatorSize      = MaxValidator
	MaxValidator           = ValidatorChunkSize * SubcircuitNumber
	TestValidatorChunkSize = ValidatorChunkSize
	// OptimalSourceSize    = MaxValidator / 256
	SubcircuitNumber            = 1 << SubcircuitExp
	SubcircuitExp               = 11
	MaxValidatorExp             = 29
	MaxValidatorShuffleRoundExp = 21 + 7
	MaxSubCircuitExp            = 12
	HalfHashOutputBitLen        = 128
)
*/
const SHUFFLE_ROUND: usize = 90;
const PREPARE_CHUNK_SIZE: usize = 2048;
const VALIDATOR_CHUNK_SIZE: usize = 128 * 2;
const TEST_VALIDATOR_SIZE: usize = MAX_VALIDATOR;
const MAX_VALIDATOR: usize = VALIDATOR_CHUNK_SIZE * SUBCIRCUIT_NUMBER;
const TEST_VALIDATOR_CHUNK_SIZE: usize = VALIDATOR_CHUNK_SIZE;
const SUBCIRCUIT_NUMBER: usize = 1 << SUBCIRCUIT_EXP;
const SUBCIRCUIT_EXP: usize = 11;
const MAX_VALIDATOR_EXP: usize = 29;
const MAX_VALIDATOR_SHUFFLE_ROUND_EXP: usize = 21 + 7;
const MAX_SUBCIRCUIT_EXP: usize = 12;
const POSEIDON_HASH_LENGTH: usize = 8;

/*
type ShuffleWithHashMapAggPubkeyCircuit struct {
	StartIndex         frontend.Variable
	ChunkLength        frontend.Variable
	ShuffleIndices     [ValidatorChunkSize]frontend.Variable `gnark:",public"`
	CommitteeIndices   [ValidatorChunkSize]frontend.Variable `gnark:",public"`
	Pivots             [ShuffleRound]frontend.Variable
	IndexCount         frontend.Variable
	PositionResults    [ShuffleRound * ValidatorChunkSize]frontend.Variable           // the curIndex -> curPosition Table
	PositionBitResults [ShuffleRound * ValidatorChunkSize]frontend.Variable           `gnark:",public"` // mimic a hint: query the positionBitSortedResults table with runtime positions, get the flip bits
	FlipResults        [ShuffleRound * ValidatorChunkSize]frontend.Variable           // mimic a hint: get the flips, but we will ensure the correctness of the flipResults in the funciton (CheckPhasesAndResults)
	ValidatorHashes    [ValidatorChunkSize][hash.PoseidonHashLength]frontend.Variable `gnark:",public"`
	Slot               frontend.Variable                                              //the pre-pre beacon root
	AggregationBits    [ValidatorChunkSize]frontend.Variable                          //the aggregation bits
	AggregatedPubkey   sw_bls12381_m31.G1Affine                                       `gnark:",public"` //the aggregated pubkey of this committee, used for later signature verification circuit
	AttestationBalance [8]frontend.Variable                                           `gnark:",public"` //the attestation balance of this committee, the accBalance of each effective attestation should be supermajority, > 2/3 total balance
}

*/
// Define defines the circuit
declare_circuit!(ShuffleWithHashMapAggPubkeyCircuit {
    start_index:         Variable,
    chunk_length:        Variable,
    shuffle_indices:     [Variable;VALIDATOR_CHUNK_SIZE],
    committee_indices:   [Variable;VALIDATOR_CHUNK_SIZE],
    pivots:             [Variable;SHUFFLE_ROUND],
    index_count:         Variable,
    position_results:    [Variable;SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE],
    position_bit_results: [Variable;SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE],
    flip_results:        [Variable;SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE],
    validator_hashes:    [[Variable;POSEIDON_HASH_LENGTH];VALIDATOR_CHUNK_SIZE],
    slot:               Variable,
    aggregation_bits:    [Variable;VALIDATOR_CHUNK_SIZE],
    aggregated_pubkey:   G1Affine,
    attestation_balance: [Variable;8],
    pubkeys_bls:      [G1Affine;VALIDATOR_CHUNK_SIZE],
    validators:      [ValidatorSSZ;VALIDATOR_CHUNK_SIZE],
});

/*
func (circuit *ShuffleWithHashMapAggPubkeyCircuit) Define(api frontend.API) error {
	logup.Reset()
	curValidatorExp := int(math.Ceil(math.Log2(float64(MaxValidator))))
	logup.NewRangeProof(curValidatorExp)

	indicesChunk := GetIndiceChunk(api, circuit.StartIndex, circuit.ChunkLength, ValidatorChunkSize)

	//set padding indices to 0
	for i := 0; i < len(indicesChunk); i++ {
		ignoreFlag := api.IsZero(api.Add(circuit.FlipResults[i], 1))
		indicesChunk[i] = api.Select(ignoreFlag, 0, indicesChunk[i])
	}
	//flip the indices based on the hashbit
	curIndices := make([]frontend.Variable, len(indicesChunk))
	copy(curIndices, indicesChunk[:])
	//flatten the loop to reduce the gkr layers
	copyCurIndices := common.CopyArray(api, curIndices)
	for i := 0; i < ShuffleRound; i++ {
		//flip the indices based on the hashbit
		curIndices = flipWithHashBits(api, circuit.Pivots[i], circuit.IndexCount, copyCurIndices, circuit.PositionResults[i*ValidatorChunkSize:(i+1)*ValidatorChunkSize], circuit.PositionBitResults[i*ValidatorChunkSize:(i+1)*ValidatorChunkSize], circuit.FlipResults[i*ValidatorChunkSize:(i+1)*ValidatorChunkSize])
		copyCurIndices = common.CopyArray(api, curIndices)
	}

	//check the final curIndices, should be equal to the shuffleIndex
	//cost: 3 * MaxValidator
	for i := 0; i < len(circuit.ShuffleIndices); i++ {
		isMinusOne := api.IsZero(api.Add(circuit.FlipResults[i], 1))
		curIndices[i] = api.Select(isMinusOne, circuit.ShuffleIndices[i], curIndices[i])
		// api.Println("ShuffleIndices", circuit.ShuffleIndices[i], curIndices[i])
		tmpRes := api.IsZero(api.Sub(circuit.ShuffleIndices[i], curIndices[i]))
		api.AssertIsEqual(tmpRes, 1)
	}

	//TODO: we need to use a lookup circuit to ensure that (shuffleIndices, committeeIndices) in the (shuffleindices, validatorIndices) table

	//at the same time, we use the circuit.CommitteeIndice (contain a committee's indices) to lookup the pubkey list
	pubkeyList, accBalance := LookupPubkeyListForCommitteeBySlot(api, circuit.CommitteeIndices[:], circuit.Slot, circuit.ValidatorHashes[:])
	//later, we may need to check the realBalance by using aggregationBits
	effectBalance := CalculateBalance(api, accBalance, circuit.AggregationBits[:])
	//make the effectBalance public
	for i := 0; i < len(effectBalance); i++ {
		api.AssertIsEqual(effectBalance[i], circuit.AttestationBalance[i])
	}

	pubkeyListBLS := make([]sw_bls12381_m31.G1Affine, len(pubkeyList))
	for i := 0; i < len(pubkeyList); i++ {
		pubkeyListBLS[i] = bls.ConvertToPublicKeyBLS(api, pubkeyList[i])
	}

	//aggregate the pubkey list
	attestation.AggregateAttestationPublicKey(api, pubkeyListBLS, circuit.AggregationBits[:], circuit.AggregatedPubkey)
	logup.FinalCheck(api, logup.ColumnCombineOption)
	// api.Println("Pass!")
	return nil
}
*/

impl GenericDefine<M31Config> for ShuffleWithHashMapAggPubkeyCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut g1 = G1::new(builder);

        let mut indices_chunk = get_indice_chunk(builder, self.start_index, self.chunk_length, VALIDATOR_CHUNK_SIZE);

        //set padding indices to 0
        let zero_var = builder.constant(0);
        for i in 0..indices_chunk.len() {
            let tmp = builder.add(self.flip_results[i], 1);
            let ignore_flag = builder.is_zero(tmp);
            indices_chunk[i] = simple_select(builder, ignore_flag, zero_var.clone(), indices_chunk[i]);
        }
        //flip the indices based on the hashbit
        let mut cur_indices = indices_chunk.clone();
        let mut copy_cur_indices = builder.new_hint("myhint.copyvarshint",  &cur_indices, cur_indices.len());
        for i in 0..SHUFFLE_ROUND {
            cur_indices = flip_with_hash_bits(builder, &mut g1.curve_f.table, self.pivots[i], self.index_count, &copy_cur_indices, &self.position_results[i*VALIDATOR_CHUNK_SIZE..(i+1)*VALIDATOR_CHUNK_SIZE], &self.position_bit_results[i*VALIDATOR_CHUNK_SIZE..(i+1)*VALIDATOR_CHUNK_SIZE], &self.flip_results[i*VALIDATOR_CHUNK_SIZE..(i+1)*VALIDATOR_CHUNK_SIZE]);
            copy_cur_indices = builder.new_hint("myhint.copyvarshint",  &cur_indices, cur_indices.len());
        }

        //check the final curIndices, should be equal to the shuffleIndex
        for i in 0..self.shuffle_indices.len() {
            let tmp = builder.add(self.flip_results[i], 1);
            let is_minus_one = builder.is_zero(tmp);
            cur_indices[i] = simple_select(builder, is_minus_one, self.shuffle_indices[i], cur_indices[i]);
            let tmp = builder.sub(self.shuffle_indices[i], cur_indices[i]);
            let tmp_res = builder.is_zero(tmp);
            builder.assert_is_equal(tmp_res, 1);
        }

        let mut pubkey_list = vec![];
        let mut acc_balance = vec![];
        for i in 0..self.committee_indices.len() {
            pubkey_list.push(self.validators[i].public_key.clone());
            acc_balance.push(self.validators[i].effective_balance.clone());
        }
        let effect_balance = calculate_balance(builder, &mut acc_balance, &self.aggregation_bits);
        for i in 0..effect_balance.len() {
            builder.assert_is_equal(effect_balance[i], self.attestation_balance[i]);
        }

        let mut pubkey_list_bls = vec![];
        for i in 0..pubkey_list.len() {
            let logup_var = check_pubkey_key_bls(builder, pubkey_list[i].to_vec(), &self.pubkeys_bls[i]);
            g1.curve_f.table.rangeproof(builder, logup_var, 5);
        }

        let mut aggregated_pubkey = G1Affine::default();
        aggregate_attestation_public_key(builder, &mut g1, &pubkey_list_bls, &self.aggregation_bits, &mut aggregated_pubkey);
        
        g1.curve_f.check_mul(builder);
        g1.curve_f.table.final_check(builder);
    }
}

fn get_indice_chunk<C: Config, B: RootAPI<C>>(builder: &mut B, start: Variable, length: Variable, max_len: usize) -> Vec<Variable> {
    let mut res = vec![];
    //M31_MOD = 2147483647
    let neg_one = builder.constant(2147483647-1);
    for i in 0..max_len {
        let tmp = builder.sub(length, i as u32);
        let reach_end = builder.is_zero(tmp);
        let mut tmp = builder.add(start, i as u32);
        tmp = simple_select(builder, reach_end, neg_one.clone(), tmp);
        res.push(tmp);
    }
    res
}
fn calculate_balance<C: Config, B: RootAPI<C>>(builder: &mut B, acc_balance: &mut Vec<[Variable;8]>, aggregation_bits: &[Variable]) -> Vec<Variable> {
    if acc_balance.len() == 0 || acc_balance[0].len() == 0 {
        panic!("accBalance is empty or invalid balance");
    } else if acc_balance.len() == 1 {
        return acc_balance[0].to_vec();
    }
    //initialize the balance
    let mut cur_balance = vec![builder.constant(0); acc_balance[0].len()];
    let zero_var = builder.constant(0);

    //set the balance to 0 if aggregationBits[i] = 0
    for i in 0..aggregation_bits.len() {
        for j in 0..acc_balance[i].len() {
            acc_balance[i][j] = simple_select(builder, aggregation_bits[i], acc_balance[i][j], zero_var.clone());
        }
    }
    //since balance is [8]frontend.Variable, we need to support Array addition
    for i in 0..acc_balance.len() {
        cur_balance = big_array_add(builder, &cur_balance, &acc_balance[i], cur_balance.len());
    }
    cur_balance
}
fn flip_with_hash_bits<C: Config, B: RootAPI<C>>(builder: &mut B, table: &mut LogUpRangeProofTable, pivot: Variable, index_count: Variable, cur_indices: &[Variable], position_results: &[Variable], position_bit_results: &[Variable], flip_results: &[Variable]) -> Vec<Variable> {
    let mut res = vec![];
    for i in 0..cur_indices.len() {
        let tmp = builder.add(flip_results[i].clone(), 1);
        let ignore_flag = builder.is_zero(tmp);
        let tmp = builder.sub(pivot, cur_indices[i]);
        let tmp = builder.sub(tmp, flip_results[i].clone());
        let flip_flag1 = builder.is_zero(tmp);
        let tmp = builder.add(index_count, pivot);
        let tmp = builder.sub(tmp, cur_indices[i]);
        let tmp = builder.sub(tmp, flip_results[i].clone());
        let flip_flag2 = builder.is_zero(tmp);
        let tmp = builder.or(flip_flag1, flip_flag2);
        let flip_flag = builder.or(tmp, ignore_flag);
        builder.assert_is_equal(flip_flag, 1);

        let tmp = builder.sub(position_results[i], flip_results[i]);
        let position_flag1 = builder.is_zero(tmp);
        let tmp = builder.sub(position_results[i], cur_indices[i]);
        let position_flag2 = builder.is_zero(tmp);
        let tmp = builder.or(position_flag1, position_flag2);
        let position_flag = builder.or(tmp, ignore_flag);
        builder.assert_is_equal(position_flag, 1);

        let tmp = builder.mul(2, position_results[i]);
        let tmp = builder.sub(tmp, flip_results[i].clone());
        let position_diff = builder.sub(tmp, cur_indices[i]);
        let zero_var = builder.constant(0);
        let position_diff = simple_select(builder, ignore_flag, zero_var.clone(), position_diff);
        table.rangeproof(builder, position_diff, MAX_VALIDATOR_EXP);

        res.push(simple_select(builder, position_bit_results[i], flip_results[i], cur_indices[i]));
    }
    res
}
pub fn aggregate_attestation_public_key<C: Config, B: RootAPI<C>>(builder: &mut B, g1: &mut G1, pub_key: &[G1Affine], validator_agg_bits: &[Variable], agg_pubkey: &mut G1Affine) {
    // let mut g1 = G1::new(builder);
    // let mut validator_bits_vec = validator_agg_bits.to_vec();
    // let mut scalar = builder.constant(0);
    // for j in 0..validator_agg_bits.len() {
    //     pub_key[j].x = g1.curve_f.select(builder, validator_agg_bits[j], &pub_key[j].x, &pub_key[0].x);
    //     pub_key[j].y = g1.curve_f.select(builder, validator_agg_bits[j], &pub_key[j].y, &pub_key[0].y);
    //     scalar = builder.add(scalar, builder.xor(validator_agg_bits[j], 1));
    // }
    // let curve = g1.curve_f.curve.clone();
    // let f = curve.fr.clone();
    // let mut scalars = vec![builder.constant(0); 32];
    // scalars[0] = scalar;
    // let scalar_element = f.new_element(&scalars);
    // let g1_minus = curve.scalar_mul(&pub_key[0], &scalar_element);

    // let mut g1_add = g1_minus.clone();
    // g1_add.y = g1.curve_f.table.neg(&g1_add.y);
    // let mut aggregated_pubkey = g1_add.clone();
    // for i in 0..validator_agg_bits.len() {
    //     aggregated_pubkey = g1.curve_f.add(&aggregated_pubkey, &pub_key[i]);
    // }
    // g1.curve_f.table.assert_is_equal(&aggregated_pubkey.x, &agg_pubkey.x);
    // g1.curve_f.table.assert_is_equal(&aggregated_pubkey.y, &agg_pubkey.y);
}

// #[test]
// fn test_shuffle_with_hash_map_agg_pubkey_circuit() {
//     let mut rng = ark_std::test_rng();
//     let mut builder = M31Config::default();
//     let mut circuit = ShuffleWithHashMapAggPubkeyCircuit {
//         start_index: builder.constant(0),
//         chunk_length: builder.constant(0),
//         shuffle_indices: [builder.constant(0); VALIDATOR_CHUNK_SIZE],
//         committee_indices: [builder.constant(0); VALIDATOR_CHUNK_SIZE],
//         pivots: [builder.constant(0); SHUFFLE_ROUND],
//         index_count: builder.constant(0),
//         position_results: [builder.constant(0); SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE],
//         position_bit_results: [builder.constant(0); SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE],
//         flip_results: [builder.constant(0); SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE],
//         validator_hashes: [[builder.constant(0); POSEIDON_HASH_LENGTH]; VALIDATOR_CHUNK_SIZE],
//         slot: builder.constant(0),
//         aggregation_bits: [builder.constant(0); VALIDATOR_CHUNK_SIZE],
//         aggregated_pubkey: G1Affine::default(),
//         attestation_balance: [builder.constant(0); 8],
//         pubkeys_bls: [G1Affine::default(); VALIDATOR_CHUNK_SIZE],
//         validators: [ValidatorSSZ::new(); VALIDATOR_CHUNK_SIZE],
//     };
//     run_circuit::<M31Config, ShuffleWithHashMapAggPubkeyCircuit<Variable>>(&mut builder, &mut circuit);
// }