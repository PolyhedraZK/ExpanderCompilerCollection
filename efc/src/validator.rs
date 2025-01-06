use std::thread;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use ark_bls12_381::g2;
use circuit_std_rs::gnark::hints::register_hint;
use circuit_std_rs::poseidon_m31::*;
use circuit_std_rs::utils::simple_select;
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

#[derive(Clone, Copy)]
pub struct ValidatorSSZ {
    pub public_key: [Variable; 48],
    pub withdrawal_credentials: [Variable; 32],
    pub effective_balance: [Variable; 8],
    pub slashed: [Variable; 1],
    pub activation_eligibility_epoch: [Variable; 8],
    pub activation_epoch: [Variable; 8],
    pub exit_epoch: [Variable; 8],
    pub withdrawable_epoch: [Variable; 8],
}
impl Default for ValidatorSSZ {
    fn default() -> Self {
        Self {
            public_key: [Variable::default(); 48],
            withdrawal_credentials: [Variable::default(); 32],
            effective_balance: [Variable::default(); 8],
            slashed: [Variable::default(); 1],
            activation_eligibility_epoch: [Variable::default(); 8],
            activation_epoch: [Variable::default(); 8],
            exit_epoch: [Variable::default(); 8],
            withdrawable_epoch: [Variable::default(); 8],
        }
    }
}
impl ValidatorSSZ {
    // pub fn default() -> Self {
    //     Self {
    //         public_key: [Variable::default(); 48],
    //         withdrawal_credentials: [Variable::default(); 32],
    //         effective_balance: [Variable::default(); 8],
    //         slashed: [Variable::default(); 1],
    //         activation_eligibility_epoch: [Variable::default(); 8],
    //         activation_epoch: [Variable::default(); 8],
    //         exit_epoch: [Variable::default(); 8],
    //         withdrawable_epoch: [Variable::default(); 8],
    //     }
    // }
    pub fn new() -> Self {
        Self {
            public_key: [Variable::default(); 48],
            withdrawal_credentials: [Variable::default(); 32],
            effective_balance: [Variable::default(); 8],
            slashed: [Variable::default(); 1],
            activation_eligibility_epoch: [Variable::default(); 8],
            activation_epoch: [Variable::default(); 8],
            exit_epoch: [Variable::default(); 8],
            withdrawable_epoch: [Variable::default(); 8],
        }
    }
    pub fn hash<C: Config, B: RootAPI<C>>(&self, builder: &mut B) -> Vec<Variable> {
        let mut inputs = Vec::new();
        for i in 0..48 {
            inputs.push(self.public_key[i]);
        }
        for i in 0..32 {
            inputs.push(self.withdrawal_credentials[i]);
        }
        for i in 0..8 {
            inputs.push(self.effective_balance[i]);
        }
        for i in 0..1 {
            inputs.push(self.slashed[i]);
        }
        for i in 0..8 {
            inputs.push(self.activation_eligibility_epoch[i]);
        }
        for i in 0..8 {
            inputs.push(self.activation_epoch[i]);
        }
        for i in 0..8 {
            inputs.push(self.exit_epoch[i]);
        }
        for i in 0..8 {
            inputs.push(self.withdrawable_epoch[i]);
        }
        let hash = poseidon_elements_hint(builder, &PoseidonParams::new(), inputs, false);
        hash
    }
}
/*

type ConvertValidatorListToMerkleTreeCircuit struct {
	ValidatorHashChunk [SUBTREESIZE][hash.PoseidonHashLength]frontend.Variable
	SubtreeRoot        [hash.PoseidonHashLength]frontend.Variable `gnark:",public"`
}

func (circuit *ConvertValidatorListToMerkleTreeCircuit) Define(api frontend.API) error {
	inputs := make([]frontend.Variable, 0)
	for i := 0; i < len(circuit.ValidatorHashChunk); i++ {
		inputs = append(inputs, circuit.ValidatorHashChunk[i][:]...)
	}
	subTreeRoot := hash.GenericHash(api, inputs, hash.PoseidonHashLength)
	for i := 0; i < hash.PoseidonHashLength; i++ {
		api.AssertIsEqual(subTreeRoot[i], circuit.SubtreeRoot[i])
	}
	return nil
}

type MerkleSubTreeWithLimitCircuit struct {
	SubtreeRoot        [SUBTREENUM][hash.PoseidonHashLength]frontend.Variable `gnark:",public"`
	TreeRootMixIn      [hash.PoseidonHashLength]frontend.Variable             `gnark:",public"`
	RealValidatorCount [8]frontend.Variable                                   `gnark:",public"` //little-endian encoding
	TreeRoot           [hash.PoseidonHashLength]frontend.Variable
	Path               [PADDINGDEPTH]frontend.Variable
	Aunts              [PADDINGDEPTH][hash.PoseidonHashLength]frontend.Variable
}

func (circuit *MerkleSubTreeWithLimitCircuit) Define(api frontend.API) error {
	inputs := make([]frontend.Variable, 0)
	for i := 0; i < len(circuit.SubtreeRoot); i++ {
		inputs = append(inputs, circuit.SubtreeRoot[i][:]...)
	}
	subTreeRootRoot := hash.GenericHash(api, inputs, hash.PoseidonHashLength)
	aunts := make([][]frontend.Variable, len(circuit.Aunts))
	for i := 0; i < len(circuit.Aunts); i++ {
		aunts[i] = make([]frontend.Variable, len(circuit.Aunts[i]))
		copy(aunts[i][:], circuit.Aunts[i][:])
	}
	//make sure the merkle tree root is correct
	merkle.VerifyMerkleTreePathVariable(api, circuit.TreeRoot[:], subTreeRootRoot, circuit.Path[:], aunts, 0)
	//calculate the treeRootMixIn, which is held by the verifier and changed every epoch
	treeRootMixIn := hash.GenericHash(api, append(circuit.TreeRoot[:], circuit.RealValidatorCount[:]...), hash.PoseidonHashLength)
	for i := 0; i < hash.PoseidonHashLength; i++ {
		api.AssertIsEqual(treeRootMixIn[i], circuit.TreeRootMixIn[i])
	}
	return nil
}

*/