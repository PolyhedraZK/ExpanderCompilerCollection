use tiny_keccak::{Hasher, Keccak};

const POSEIDON_SEED_PREFIX: &str = "poseidon_seed";

const FIELD_NAME: &str = "Mersenne 31";

pub fn get_constants(width: usize, round_num: usize) -> Vec<Vec<u32>> {
    let seed = format!("{POSEIDON_SEED_PREFIX}_{}_{}", FIELD_NAME, width);

    let mut keccak = Keccak::v256();
    let mut buffer = [0u8; 32];
    keccak.update(seed.as_bytes());
    keccak.finalize(&mut buffer);

    let mut res = vec![vec![0u32; width]; round_num];

    (0..round_num).for_each(|i| {
        (0..width).for_each(|j| {
            let mut keccak = Keccak::v256();
            keccak.update(&buffer);
            keccak.finalize(&mut buffer);

            let mut u32_le_bytes = [0u8; 4];
            u32_le_bytes.copy_from_slice(&buffer[..4]);

            res[i][j] = u32::from_le_bytes(u32_le_bytes);
        });
    });

    res
}

const MATRIX_CIRC_MDS_8_SML_ROW: [u32; 8] = [7, 1, 3, 8, 8, 3, 4, 9];

const MATRIX_CIRC_MDS_12_SML_ROW: [u32; 12] = [1, 1, 2, 1, 8, 9, 10, 7, 5, 9, 4, 10];

const MATRIX_CIRC_MDS_16_SML_ROW: [u32; 16] =
    [1, 1, 51, 1, 11, 17, 2, 1, 101, 63, 15, 2, 67, 22, 13, 3];

pub fn get_mds_matrix(width: usize) -> Vec<Vec<u32>> {
    let mds_first_row: &[u32] = match width {
        8 => &MATRIX_CIRC_MDS_8_SML_ROW,
        12 => &MATRIX_CIRC_MDS_12_SML_ROW,
        16 => &MATRIX_CIRC_MDS_16_SML_ROW,
        _ => panic!("unsupported state width for MDS matrix"),
    };

    let mut res = vec![vec![0u32; width]; width];

    (0..width).for_each(|i| (0..width).for_each(|j| res[i][j] = mds_first_row[(i + j) % width]));

    res
}