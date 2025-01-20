use circuit_std_rs::poseidon_m31::*;
use circuit_std_rs::utils::register_hint;
use expander_compiler::frontend::*;
use extra::debug_eval;
use serde::Deserialize;

use efc::utils::read_from_json_file;

#[derive(Debug, Deserialize, Clone)]
pub struct ValidatorPlain {
    #[serde(default)]
    pub public_key: String,
    #[serde(default)]
    pub withdrawal_credentials: String,
    #[serde(default)]
    pub effective_balance: u64,
    #[serde(default)]
    pub slashed: bool,
    #[serde(default)]
    pub activation_eligibility_epoch: u64,
    #[serde(default)]
    pub activation_epoch: u64,
    #[serde(default)]
    pub exit_epoch: u64,
    #[serde(default)]
    pub withdrawable_epoch: u64,
}
pub fn read_validators(dir: &str) -> Vec<ValidatorPlain> {
    let file_path = format!("{}/validatorList.json", dir);
    let validaotrs: Vec<ValidatorPlain> = read_from_json_file(&file_path).unwrap();
    validaotrs
}

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
        let params = PoseidonM31Params::new(
            builder,
            POSEIDON_M31X16_RATE,
            16,
            POSEIDON_M31X16_FULL_ROUNDS,
            POSEIDON_M31X16_PARTIAL_ROUNDS,
        );
        params.hash_to_state(builder, &inputs)
    }
}

declare_circuit!(ValidatorSSZCircuit {
    public_key: [Variable; 48],
    withdrawal_credentials: [Variable; 32],
    effective_balance: [Variable; 8],
    slashed: [Variable; 1],
    activation_eligibility_epoch: [Variable; 8],
    activation_epoch: [Variable; 8],
    exit_epoch: [Variable; 8],
    withdrawable_epoch: [Variable; 8],
    hash: [Variable; 8],
});

impl GenericDefine<M31Config> for ValidatorSSZCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut validator = ValidatorSSZ::new();
        for i in 0..48 {
            validator.public_key[i] = self.public_key[i];
        }
        for i in 0..32 {
            validator.withdrawal_credentials[i] = self.withdrawal_credentials[i];
        }
        for i in 0..8 {
            validator.effective_balance[i] = self.effective_balance[i];
        }
        for i in 0..1 {
            validator.slashed[i] = self.slashed[i];
        }
        for i in 0..8 {
            validator.activation_eligibility_epoch[i] = self.activation_eligibility_epoch[i];
        }
        for i in 0..8 {
            validator.activation_epoch[i] = self.activation_epoch[i];
        }
        for i in 0..8 {
            validator.exit_epoch[i] = self.exit_epoch[i];
        }
        for i in 0..8 {
            validator.withdrawable_epoch[i] = self.withdrawable_epoch[i];
        }
        let hash = validator.hash(builder);
        for i in 0..8 {
            builder.assert_is_equal(&hash[i], &self.hash[i]);
        }
    }
}
#[test]
fn test_validator_hash() {
    let mut hint_registry = HintRegistry::<M31>::new();
    register_hint(&mut hint_registry);
    let mut assignment = ValidatorSSZCircuit::<M31>::default();
    /*
    validatorSSZ {[145 100 40 136 97 61 206 231 119 13 163 28 32 34 38 131 164 66 107 73 64 74 242 209 157 88 96 20 112 101 90 87 107 84 92 193 202 86 150 161 36 253 88 137 16 180 8 6] [0 66 206 99 147 246 199 124 21 214 208 187 88 176 208 167 21 244 155 148 36 32 225 236 224 248 227 109 68 1 77 223] 32000000000 0 81250 81262 18446744073709551615 18446744073709551615}
     */
    let public_key = [
        145, 100, 40, 136, 97, 61, 206, 231, 119, 13, 163, 28, 32, 34, 38, 131, 164, 66, 107, 73,
        64, 74, 242, 209, 157, 88, 96, 20, 112, 101, 90, 87, 107, 84, 92, 193, 202, 86, 150, 161,
        36, 253, 88, 137, 16, 180, 8, 6,
    ];
    let withdrawal_credentials = [
        0, 66, 206, 99, 147, 246, 199, 124, 21, 214, 208, 187, 88, 176, 208, 167, 21, 244, 155,
        148, 36, 32, 225, 236, 224, 248, 227, 109, 68, 1, 77, 223,
    ];
    let effective_balance: u64 = 32000000000;
    let effective_balance = effective_balance.to_le_bytes();
    let slashed = 0;
    let activation_eligibility_epoch: u64 = 81250;
    let activation_eligibility_epoch = activation_eligibility_epoch.to_le_bytes();
    let activation_epoch: u64 = 81262;
    let activation_epoch = activation_epoch.to_le_bytes();
    let exit_epoch: u64 = 18446744073709551615;
    let exit_epoch = exit_epoch.to_le_bytes();
    let withdrawable_epoch: u64 = 18446744073709551615;
    let withdrawable_epoch = withdrawable_epoch.to_le_bytes();
    for i in 0..48 {
        assignment.public_key[i] = M31::from(public_key[i]);
    }
    for i in 0..32 {
        assignment.withdrawal_credentials[i] = M31::from(withdrawal_credentials[i]);
    }
    for i in 0..8 {
        assignment.effective_balance[i] = M31::from(effective_balance[i] as u32);
    }
    assignment.slashed[0] = M31::from(slashed);
    for i in 0..8 {
        assignment.activation_eligibility_epoch[i] =
            M31::from(activation_eligibility_epoch[i] as u32);
    }
    for i in 0..8 {
        assignment.activation_epoch[i] = M31::from(activation_epoch[i] as u32);
    }
    for i in 0..8 {
        assignment.exit_epoch[i] = M31::from(exit_epoch[i] as u32);
    }
    for i in 0..8 {
        assignment.withdrawable_epoch[i] = M31::from(withdrawable_epoch[i] as u32);
    }
    //[582874236 1259527646 662790355 847738717 917516425 652946882 1385777334 1053741140]
    let hash = [
        413207791, 2062085079, 636880802, 1836088159, 1440531472, 1618643357, 1733638466,
        2009577726,
    ];
    for i in 0..8 {
        assignment.hash[i] = M31::from(hash[i] as u32);
    }
    debug_eval(&ValidatorSSZCircuit::default(), &assignment, hint_registry);
}
