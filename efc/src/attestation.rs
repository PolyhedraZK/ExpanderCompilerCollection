use circuit_std_rs::sha256::m31::sha256_var_bytes;
use expander_compiler::frontend::*;
use serde::Deserialize;

const ZERO_HASHES: [&[u8]; 40] = [
    &[0; 32],
    &[
        245, 165, 253, 66, 209, 106, 32, 48, 39, 152, 239, 110, 211, 9, 151, 155, 67, 0, 61, 35,
        32, 217, 240, 232, 234, 152, 49, 169, 39, 89, 251, 75,
    ],
    &[
        219, 86, 17, 78, 0, 253, 212, 193, 248, 92, 137, 43, 243, 90, 201, 168, 146, 137, 170, 236,
        177, 235, 208, 169, 108, 222, 96, 106, 116, 139, 93, 113,
    ],
    &[
        199, 128, 9, 253, 240, 127, 197, 106, 17, 241, 34, 55, 6, 88, 163, 83, 170, 165, 66, 237,
        99, 228, 76, 75, 193, 95, 244, 205, 16, 90, 179, 60,
    ],
    &[
        83, 109, 152, 131, 127, 45, 209, 101, 165, 93, 94, 234, 233, 20, 133, 149, 68, 114, 213,
        111, 36, 109, 242, 86, 191, 60, 174, 25, 53, 42, 18, 60,
    ],
    &[
        158, 253, 224, 82, 170, 21, 66, 159, 174, 5, 186, 212, 208, 177, 215, 198, 77, 166, 77, 3,
        215, 161, 133, 74, 88, 140, 44, 184, 67, 12, 13, 48,
    ],
    &[
        216, 141, 223, 238, 212, 0, 168, 117, 85, 150, 178, 25, 66, 193, 73, 126, 17, 76, 48, 46,
        97, 24, 41, 15, 145, 230, 119, 41, 118, 4, 31, 161,
    ],
    &[
        135, 235, 13, 219, 165, 126, 53, 246, 210, 134, 103, 56, 2, 164, 175, 89, 117, 226, 37, 6,
        199, 207, 76, 100, 187, 107, 229, 238, 17, 82, 127, 44,
    ],
    &[
        38, 132, 100, 118, 253, 95, 197, 74, 93, 67, 56, 81, 103, 201, 81, 68, 242, 100, 63, 83,
        60, 200, 91, 185, 209, 107, 120, 47, 141, 125, 177, 147,
    ],
    &[
        80, 109, 134, 88, 45, 37, 36, 5, 184, 64, 1, 135, 146, 202, 210, 191, 18, 89, 241, 239, 90,
        165, 248, 135, 225, 60, 178, 240, 9, 79, 81, 225,
    ],
    &[
        255, 255, 10, 215, 230, 89, 119, 47, 149, 52, 193, 149, 200, 21, 239, 196, 1, 78, 241, 225,
        218, 237, 68, 4, 192, 99, 133, 209, 17, 146, 233, 43,
    ],
    &[
        108, 240, 65, 39, 219, 5, 68, 28, 216, 51, 16, 122, 82, 190, 133, 40, 104, 137, 14, 67, 23,
        230, 160, 42, 180, 118, 131, 170, 117, 150, 66, 32,
    ],
    &[
        183, 208, 95, 135, 95, 20, 0, 39, 239, 81, 24, 162, 36, 123, 187, 132, 206, 143, 47, 15,
        17, 35, 98, 48, 133, 218, 247, 150, 12, 50, 159, 95,
    ],
    &[
        223, 106, 245, 245, 187, 219, 107, 233, 239, 138, 166, 24, 228, 191, 128, 115, 150, 8, 103,
        23, 30, 41, 103, 111, 139, 40, 77, 234, 106, 8, 168, 94,
    ],
    &[
        181, 141, 144, 15, 94, 24, 46, 60, 80, 239, 116, 150, 158, 161, 108, 119, 38, 197, 73, 117,
        124, 194, 53, 35, 195, 105, 88, 125, 167, 41, 55, 132,
    ],
    &[
        212, 154, 117, 2, 255, 207, 176, 52, 11, 29, 120, 133, 104, 133, 0, 202, 48, 129, 97, 167,
        249, 107, 98, 223, 157, 8, 59, 113, 252, 200, 242, 187,
    ],
    &[
        143, 230, 177, 104, 146, 86, 192, 211, 133, 244, 47, 91, 190, 32, 39, 162, 44, 25, 150,
        225, 16, 186, 151, 193, 113, 211, 229, 148, 141, 233, 43, 235,
    ],
    &[
        141, 13, 99, 195, 158, 186, 222, 133, 9, 224, 174, 60, 156, 56, 118, 251, 95, 161, 18, 190,
        24, 249, 5, 236, 172, 254, 203, 146, 5, 118, 3, 171,
    ],
    &[
        149, 238, 200, 178, 229, 65, 202, 212, 233, 29, 227, 131, 133, 242, 224, 70, 97, 159, 84,
        73, 108, 35, 130, 203, 108, 172, 213, 185, 140, 38, 245, 164,
    ],
    &[
        248, 147, 233, 8, 145, 119, 117, 182, 43, 255, 35, 41, 77, 187, 227, 161, 205, 142, 108,
        193, 195, 91, 72, 1, 136, 123, 100, 106, 111, 129, 241, 127,
    ],
    &[
        205, 219, 167, 181, 146, 227, 19, 51, 147, 193, 97, 148, 250, 199, 67, 26, 191, 47, 84,
        133, 237, 113, 29, 178, 130, 24, 60, 129, 158, 8, 235, 170,
    ],
    &[
        138, 141, 127, 227, 175, 140, 170, 8, 90, 118, 57, 168, 50, 0, 20, 87, 223, 185, 18, 138,
        128, 97, 20, 42, 208, 51, 86, 41, 255, 35, 255, 156,
    ],
    &[
        254, 179, 195, 55, 215, 165, 26, 111, 191, 0, 185, 227, 76, 82, 225, 201, 25, 92, 150, 155,
        212, 231, 160, 191, 213, 29, 92, 91, 237, 156, 17, 103,
    ],
    &[
        231, 31, 10, 168, 60, 195, 46, 223, 190, 250, 159, 77, 62, 1, 116, 202, 133, 24, 46, 236,
        159, 58, 9, 246, 166, 192, 223, 99, 119, 165, 16, 215,
    ],
    &[
        49, 32, 111, 168, 10, 80, 187, 106, 190, 41, 8, 80, 88, 241, 98, 18, 33, 42, 96, 238, 200,
        240, 73, 254, 203, 146, 216, 200, 224, 168, 75, 192,
    ],
    &[
        33, 53, 43, 254, 203, 237, 221, 233, 147, 131, 159, 97, 76, 61, 172, 10, 62, 227, 117, 67,
        249, 180, 18, 177, 97, 153, 220, 21, 142, 35, 181, 68,
    ],
    &[
        97, 158, 49, 39, 36, 187, 109, 124, 49, 83, 237, 157, 231, 145, 215, 100, 163, 102, 179,
        137, 175, 19, 197, 139, 248, 168, 217, 4, 129, 164, 103, 101,
    ],
    &[
        124, 221, 41, 134, 38, 130, 80, 98, 141, 12, 16, 227, 133, 197, 140, 97, 145, 230, 251,
        224, 81, 145, 188, 192, 79, 19, 63, 44, 234, 114, 193, 196,
    ],
    &[
        132, 137, 48, 189, 123, 168, 202, 197, 70, 97, 7, 33, 19, 251, 39, 136, 105, 224, 123, 184,
        88, 127, 145, 57, 41, 51, 55, 77, 1, 123, 203, 225,
    ],
    &[
        136, 105, 255, 44, 34, 178, 140, 193, 5, 16, 217, 133, 50, 146, 128, 51, 40, 190, 79, 176,
        232, 4, 149, 232, 187, 141, 39, 31, 91, 136, 150, 54,
    ],
    &[
        181, 254, 40, 231, 159, 27, 133, 15, 134, 88, 36, 108, 233, 182, 161, 231, 180, 159, 192,
        109, 183, 20, 62, 143, 224, 180, 242, 176, 197, 82, 58, 92,
    ],
    &[
        152, 94, 146, 159, 112, 175, 40, 208, 189, 209, 169, 10, 128, 143, 151, 127, 89, 124, 124,
        119, 140, 72, 158, 152, 211, 189, 137, 16, 211, 26, 192, 247,
    ],
    &[
        198, 246, 126, 2, 230, 228, 225, 189, 239, 185, 148, 198, 9, 137, 83, 243, 70, 54, 186, 43,
        108, 162, 10, 71, 33, 210, 178, 106, 136, 103, 34, 255,
    ],
    &[
        28, 154, 126, 95, 241, 207, 72, 180, 173, 21, 130, 211, 244, 228, 161, 0, 79, 59, 32, 216,
        197, 162, 183, 19, 135, 164, 37, 74, 217, 51, 235, 197,
    ],
    &[
        47, 7, 90, 226, 41, 100, 107, 111, 106, 237, 25, 165, 227, 114, 207, 41, 80, 129, 64, 30,
        184, 147, 255, 89, 155, 63, 154, 204, 12, 13, 62, 125,
    ],
    &[
        50, 137, 33, 222, 181, 150, 18, 7, 104, 1, 232, 205, 97, 89, 33, 7, 181, 198, 124, 121,
        184, 70, 89, 92, 198, 50, 12, 57, 91, 70, 54, 44,
    ],
    &[
        191, 185, 9, 253, 178, 54, 173, 36, 17, 180, 228, 136, 56, 16, 160, 116, 184, 64, 70, 70,
        137, 152, 108, 63, 138, 128, 145, 130, 126, 23, 195, 39,
    ],
    &[
        85, 216, 251, 54, 135, 186, 59, 164, 159, 52, 44, 119, 245, 161, 248, 155, 236, 131, 216,
        17, 68, 110, 26, 70, 113, 57, 33, 61, 100, 11, 106, 116,
    ],
    &[
        247, 33, 13, 79, 142, 126, 16, 57, 121, 14, 123, 244, 239, 162, 7, 85, 90, 16, 166, 219,
        29, 212, 185, 93, 163, 19, 170, 168, 139, 136, 254, 118,
    ],
    &[
        173, 33, 181, 22, 203, 198, 69, 255, 227, 74, 181, 222, 28, 138, 239, 140, 212, 231, 248,
        210, 181, 30, 142, 20, 86, 173, 199, 86, 60, 218, 32, 111,
    ],
];

#[derive(Debug, Deserialize, Clone)]
pub struct CheckpointPlain {
    pub epoch: u64,
    pub root: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct AttestationData {
    #[serde(default)]
    pub slot: u64,
    #[serde(default)]
    pub committee_index: u64,
    pub beacon_block_root: String,
    pub source: CheckpointPlain,
    pub target: CheckpointPlain,
}
#[derive(Debug, Deserialize, Clone)]
pub struct Attestation {
    #[serde(default)]
    pub aggregation_bits: String,
    pub data: AttestationData,
    pub signature: String,
}

#[derive(Default, Clone, Copy)]
pub struct AttestationDataSSZ {
    pub slot: [Variable; 8],
    pub committee_index: [Variable; 8],
    pub beacon_block_root: [Variable; 32],
    pub source_epoch: [Variable; 8],
    pub target_epoch: [Variable; 8],
    pub source_root: [Variable; 32],
    pub target_root: [Variable; 32],
}
impl AttestationDataSSZ {
    pub fn new() -> Self {
        Self {
            slot: [Variable::default(); 8],
            committee_index: [Variable::default(); 8],
            beacon_block_root: [Variable::default(); 32],
            source_epoch: [Variable::default(); 8],
            target_epoch: [Variable::default(); 8],
            source_root: [Variable::default(); 32],
            target_root: [Variable::default(); 32],
        }
    }
    pub fn att_data_signing_root<C: Config, B: RootAPI<C>>(
        &self,
        builder: &mut B,
        att_domain: &[Variable],
    ) -> Vec<Variable> {
        let att_data_hash_tree_root = self.hash_tree_root(builder);
        bytes_hash_tree_root(
            builder,
            [att_data_hash_tree_root, att_domain.to_vec()].concat(),
        )
    }

    pub fn check_point_hash_tree_variable<C: Config, B: RootAPI<C>>(
        &self,
        builder: &mut B,
        epoch: &[Variable],
        root: &[Variable],
    ) -> Vec<Variable> {
        let mut inputs = Vec::new();
        inputs.extend_from_slice(&append_to_32_bytes(builder, epoch));
        inputs.extend_from_slice(root);
        bytes_hash_tree_root(builder, inputs)
    }
    pub fn hash_tree_root<C: Config, B: RootAPI<C>>(&self, builder: &mut B) -> Vec<Variable> {
        let mut inputs = Vec::new();
        inputs.extend_from_slice(&append_to_32_bytes(builder, &self.slot));
        inputs.extend_from_slice(&append_to_32_bytes(builder, &self.committee_index));
        inputs.extend_from_slice(&self.beacon_block_root);
        let source_checkpoint_root =
            self.check_point_hash_tree_variable(builder, &self.source_epoch, &self.source_root);
        inputs.extend_from_slice(&source_checkpoint_root);
        let target_checkpoint_root =
            self.check_point_hash_tree_variable(builder, &self.target_epoch, &self.target_root);
        inputs.extend_from_slice(&target_checkpoint_root);
        bytes_hash_tree_root(builder, inputs)
    }
}
pub fn bytes_hash_tree_root<C: Config, B: RootAPI<C>>(
    builder: &mut B,
    inputs: Vec<Variable>,
) -> Vec<Variable> {
    let chunks = to_chunks(&append_to_32_bytes(builder, &inputs));
    beacon_merklize(builder, chunks).unwrap()
}
pub fn beacon_merklize<C: Config, B: RootAPI<C>>(
    builder: &mut B,
    inputs: Vec<Vec<Variable>>,
) -> Result<Vec<Variable>, String> {
    if inputs.is_empty() {
        return Err("no inputs".to_string());
    }
    if inputs.len() == 1 {
        return Ok(inputs[0].clone());
    }
    let mut length = inputs.len();
    let depth = (length as f64).log2().ceil() as usize;
    let mut inputs = inputs;
    for padding_hash in ZERO_HASHES.iter().take(depth) {
        if inputs.len() % 2 == 1 {
            let pad_hash = *padding_hash;
            let padding: Vec<_> = pad_hash
                .iter()
                .map(|&x| builder.constant(x as u32))
                .collect();
            inputs.push(padding);
        }
        let mut new_level = Vec::new();
        for j in (0..length).step_by(2) {
            let mut combined = vec![];
            combined.extend_from_slice(&inputs[j]);
            combined.extend_from_slice(&inputs[j + 1]);
            let hash = sha256_var_bytes(builder, &combined);
            new_level.push(hash);
        }
        inputs = new_level;
        length = inputs.len();
    }
    Ok(inputs[0].clone())
}

pub fn append_to_32_bytes<C: Config, B: RootAPI<C>>(
    builder: &mut B,
    input: &[Variable],
) -> Vec<Variable> {
    let rest = input.len() % 32;
    if rest != 0 {
        let padding = vec![builder.constant(0); 32 - rest];
        let mut input = input.to_vec();
        input.extend_from_slice(&padding);
        input
    } else {
        input.to_vec()
    }
}

pub fn to_chunks(input: &[Variable]) -> Vec<Vec<Variable>> {
    if input.len() % 32 != 0 {
        panic!("input length is not a multiple of 32");
    }
    input.chunks(32).map(|x| x.to_vec()).collect()
}

declare_circuit!(AttHashCircuit {
    //AttestationSSZ
    slot: [Variable; 8],
    committee_index: [Variable; 8],
    beacon_beacon_block_root: [Variable; 32],
    source_epoch: [Variable; 8],
    target_epoch: [Variable; 8],
    source_root: [Variable; 32],
    target_root: [Variable; 32],
    //att_domain
    domain: [Variable; 32],
    //att_signing_hash
    outputs: [Variable; 32],
});

impl GenericDefine<M31Config> for AttHashCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let att_ssz = AttestationDataSSZ {
            slot: self.slot,
            committee_index: self.committee_index,
            beacon_block_root: self.beacon_beacon_block_root,
            source_epoch: self.source_epoch,
            target_epoch: self.target_epoch,
            source_root: self.source_root,
            target_root: self.target_root,
        };
        let att_hash = att_ssz.att_data_signing_root(builder, &self.domain);
        for (i, att_hash_byte) in att_hash.iter().enumerate().take(32) {
            builder.assert_is_equal(att_hash_byte, self.outputs[i]);
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::{attestation::Attestation, utils::read_from_json_file};

    use super::AttHashCircuit;
    use circuit_std_rs::utils::register_hint;
    use expander_compiler::frontend::*;
    use extra::debug_eval;
    #[test]
    fn test_attestation_hash() {
        // att.Data.Slot 9280000
        // att.Data.CommitteeIndex 0
        // att.Data.BeaconBlockRoot [31 28 22 87 106 251 75 169 100 167 224 201 6 63 144 105 213 235 18 224 169 157 122 56 47 48 28 31 124 69 38 248]
        // att.Data.Source 289999 [194 212 152 232 56 145 101 103 73 230 240 242 89 129 63 184 38 157 86 185 251 148 157 68 227 144 241 74 228 200 206 199]
        // att.Data.Target 290000 [31 28 22 87 106 251 75 169 100 167 224 201 6 63 144 105 213 235 18 224 169 157 122 56 47 48 28 31 124 69 38 248]
        // att.Signature [170 121 191 2 187 22 51 113 109 233 89 181 237 140 207 117 72 230 115 61 124 161 23 145 241 245 211 134 175 182 206 188 124 240 51 154 121 27 217 24 126 83 70 24 90 206 50 148 2 182 65 209 6 215 131 231 254 32 229 193 207 91 52 22 89 10 212 80 4 160 179 150 246 97 120 81 28 231 36 195 223 118 194 250 230 31 182 130 163 236 45 222 26 229 163 89]
        // msg: [21 43 211 145 56 110 228 123 66 36 151 4 255 189 148 168 249 77 23 127 110 62 89 50 240 62 155 2 139 217 153 140]
        // domain: [1 0 0 0 187 164 218 150 53 76 159 37 71 108 241 188 105 191 88 58 127 158 10 240 73 48 91 98 222 103 102 64]
        // msgList[ 0 ]: [108 128 22 84 10 154 231 122 105 134 112 241 41 75 92 55 89 54 23 5 113 63 35 4 32 197 151 179 250 27 66 13]
        // sigList[ 0 ]: E([417406042303837766676050444382954581819710384023930335899613364000243943316124744931107291428889984115562657456985+1612337918776384379710682981548399375489832112491603419994252758241488024847803823620674751718035900645102653944468*u,2138372746384454686692156684769748785619173944336480358459807585988147682623523096063056865298570471165754367761702+2515621099638397509480666850964364949449167540660259026336903510150090825582288208580180650995842554224706524936338*u])

        let mut hint_registry = HintRegistry::<M31>::new();
        register_hint(&mut hint_registry);
        let mut assignment = AttHashCircuit::<M31> {
            slot: [M31::from(0); 8],
            committee_index: [M31::from(0); 8],
            beacon_beacon_block_root: [M31::from(0); 32],
            source_epoch: [M31::from(0); 8],
            target_epoch: [M31::from(0); 8],
            source_root: [M31::from(0); 32],
            target_root: [M31::from(0); 32],
            domain: [M31::from(0); 32],
            outputs: [M31::from(0); 32],
        };
        let slot: u64 = 9280000;
        let slot = slot.to_le_bytes();
        let committee_index: u64 = 0;
        let committee_index = committee_index.to_le_bytes();
        let beacon_beacon_block_root = vec![
            31, 28, 22, 87, 106, 251, 75, 169, 100, 167, 224, 201, 6, 63, 144, 105, 213, 235, 18,
            224, 169, 157, 122, 56, 47, 48, 28, 31, 124, 69, 38, 248,
        ];
        let source_epoch: u64 = 289999;
        let source_epoch = source_epoch.to_le_bytes();
        let target_epoch: u64 = 290000;
        let target_epoch = target_epoch.to_le_bytes();
        let source_root = vec![
            194, 212, 152, 232, 56, 145, 101, 103, 73, 230, 240, 242, 89, 129, 63, 184, 38, 157,
            86, 185, 251, 148, 157, 68, 227, 144, 241, 74, 228, 200, 206, 199,
        ];
        let target_root = vec![
            31, 28, 22, 87, 106, 251, 75, 169, 100, 167, 224, 201, 6, 63, 144, 105, 213, 235, 18,
            224, 169, 157, 122, 56, 47, 48, 28, 31, 124, 69, 38, 248,
        ];
        let domain = vec![
            1, 0, 0, 0, 187, 164, 218, 150, 53, 76, 159, 37, 71, 108, 241, 188, 105, 191, 88, 58,
            127, 158, 10, 240, 73, 48, 91, 98, 222, 103, 102, 64,
        ];
        let output = vec![
            108, 128, 22, 84, 10, 154, 231, 122, 105, 134, 112, 241, 41, 75, 92, 55, 89, 54, 23, 5,
            113, 63, 35, 4, 32, 197, 151, 179, 250, 27, 66, 13,
        ];

        for i in 0..8 {
            assignment.slot[i] = M31::from(slot[i] as u32);
            assignment.committee_index[i] = M31::from(committee_index[i] as u32);
            assignment.source_epoch[i] = M31::from(source_epoch[i] as u32);
            assignment.target_epoch[i] = M31::from(target_epoch[i] as u32);
        }
        for i in 0..32 {
            assignment.beacon_beacon_block_root[i] = M31::from(beacon_beacon_block_root[i] as u32);
            assignment.source_root[i] = M31::from(source_root[i] as u32);
            assignment.target_root[i] = M31::from(target_root[i] as u32);
            assignment.domain[i] = M31::from(domain[i] as u32);
            assignment.outputs[i] = M31::from(output[i] as u32);
        }

        debug_eval(&AttHashCircuit::default(), &assignment, hint_registry);
    }

    #[test]
    fn read_attestation() {
        let file_path = "./data/slotAttestationsFolded.json";
        let attestations: Vec<Attestation> = read_from_json_file(file_path).unwrap();
        println!("attestations[0]:{:?}", attestations[0]);
    }
}
