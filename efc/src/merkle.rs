use byteorder::{LittleEndian, WriteBytesExt};
use circuit_std_rs::{
    poseidon::{
        poseidon_m31::PoseidonM31Params, poseidon_u32::PoseidonParams, utils::POSEIDON_M31X16_RATE,
    },
    sha256,
    utils::simple_select,
};
use expander_compiler::frontend::{Config, RootAPI, Variable};
pub const MAX_BEACON_VALIDATOR_DEPTH: usize = 40;
const ZERO_HASHES_MAX_DEPTH: usize = MAX_BEACON_VALIDATOR_DEPTH;
pub const ZERO_HASHES_POSEIDON: [&[u32]; ZERO_HASHES_MAX_DEPTH] = [
    &[0, 0, 0, 0, 0, 0, 0, 0], 
    &[1479731193, 675523649, 2589942, 996409316, 662065262, 1747716529, 2069769266, 80342673], 
    &[821396457, 1128750702, 978398505, 1352502482, 1551208971, 1099393284, 2104539908, 920193086], 
    &[906167701, 720475850, 555628833, 1809438733, 899846427, 1562688178, 692056545, 1127366018], 
    &[726382661, 834602172, 1554134445, 829382637, 1400457597, 1149075447, 659442082, 965074680], 
    &[1543684731, 485430193, 1291884741, 180370626, 1729966942, 286184815, 991450236, 785507902], 
    &[2138463404, 375531849, 517771586, 6040809, 1923480582, 1946867885, 1653554274, 1909543360], 
    &[370576977, 1175115161, 1840502958, 1498812113, 1527591671, 1922432546, 513443666, 788666135], 
    &[1100461123, 1173764538, 1458453802, 684071368, 654347003, 1957593104, 1314704246, 300920764], 
    &[584656204, 703677484, 1044648157, 1562780854, 1029150139, 496594572, 1749722005, 1339904780], 
    &[2107068781, 1337351726, 273164162, 183675829, 1742571202, 529121188, 1751526292, 562852884], 
    &[567802521, 1152875818, 423332876, 999668124, 1717676214, 2015397435, 231471571, 1821536932], 
    &[1665890452, 1565589231, 991472188, 394924809, 164812360, 1608709811, 1104327347, 1719052975], 
    &[110261880, 1496972477, 1148153082, 944633513, 729767069, 2008095874, 990611941, 1020320827], 
    &[1878242275, 732801519, 717047776, 930545313, 461230325, 514404082, 1419521821, 1910809089], 
    &[848073867, 1138175758, 2078073531, 424543382, 1702729281, 1801387593, 838106620, 1417591607], 
    &[1400117053, 1187826170, 1817951944, 1948085560, 137240011, 559940626, 1385390531, 1299387419], 
    &[1140551700, 120454263, 723061353, 322071272, 322076091, 683599150, 1077448401, 680492779], 
    &[715928888, 2076409604, 1509441761, 448945431, 529621759, 1458638617, 1740807262, 1579521761], 
    &[2052417662, 1004871942, 170886004, 1520846928, 1624426175, 767683810, 194453512, 1652283486], 
    &[661733159, 1606916138, 335498488, 1812458856, 605641527, 373719753, 1769408096, 1466714296], 
    &[291903937, 342338853, 136229108, 1344476326, 190080617, 968805772, 1016742537, 1821104575], 
    &[443079460, 870583813, 1558883755, 1716837178, 771842406, 163795425, 487508590, 2124003456], 
    &[1104900632, 576648260, 352050013, 66345077, 556999631, 1466890024, 111300893, 1444197729], 
    &[1285915585, 676487619, 259207145, 1200329727, 26451210, 201749607, 989244804, 1498396908], 
    &[1640633799, 1177227291, 700748107, 1408925025, 1591794622, 1911595972, 1518230536, 1010769900], 
    &[1541691565, 370383790, 1199161582, 34072906, 1586031351, 1897418211, 288603768, 164758305], 
    &[1022857783, 187093956, 1421877890, 1503720398, 499034881, 290331629, 1905846593, 1384806329], 
    &[848898539, 684314868, 108997564, 1555507950, 876838784, 562963179, 1981877979, 355576768], 
    &[1437832680, 163984455, 1010222610, 632255922, 2121220281, 292143573, 680848813, 1393320034], 
    &[552690520, 301342655, 260143219, 96122727, 55888426, 488461917, 1050307697, 910906899], 
    &[2040916817, 2036956776, 895939545, 905662338, 1910986940, 1043517812, 816124360, 1674207451], 
    &[1409760700, 1803210018, 756773905, 395277972, 1802010456, 1584997044, 1952772766, 421218070], 
    &[781136465, 49945429, 1653854061, 1793445990, 1142445457, 533741311, 69706783, 1482551929], 
    &[1606143159, 980822805, 1076873699, 1012213782, 928836321, 1870660226, 34945608, 431310191], 
    &[1218926004, 760719128, 1870525219, 956355995, 1056040991, 831130090, 613990550, 1273718204], 
    &[1169724221, 352676058, 1561286993, 1014278503, 817874424, 492460229, 1727665525, 9295518], 
    &[33065465, 1775641695, 1305050285, 2038048132, 53208280, 218781572, 1202878899, 1353129021], 
    &[1180415187, 2017991054, 308587294, 207009032, 1456059267, 2041063345, 1182897744, 1709394793], 
    &[308121029, 1437195560, 1686917159, 2063548352, 1307388770, 2007551481, 81797277, 131807820]
];
pub fn generate_zero_hashes_poseidon(param: &PoseidonParams) -> Vec<Vec<u32>> {
    let mut zero_hashes_poseidon: Vec<Vec<u32>> =
        vec![vec![0u32; POSEIDON_M31X16_RATE]; ZERO_HASHES_MAX_DEPTH];

    for i in 1..ZERO_HASHES_MAX_DEPTH {
        let left = &zero_hashes_poseidon[i - 1];
        let right = &zero_hashes_poseidon[i - 1];
        let mut combined = left.clone();
        combined.extend_from_slice(right);

        let res = param.hash(&combined); // Assume this function is defined
        zero_hashes_poseidon[i] = res;
    }

    zero_hashes_poseidon
}
#[test]
fn test_zero_hashes_poseidon() {
    let param = PoseidonParams::new(
        POSEIDON_M31X16_RATE,
        16,
        8,
        14,
    );
    let zero_hashes_poseidon = generate_zero_hashes_poseidon(&param);
    println!("{:?}", zero_hashes_poseidon);
    for i in 0..ZERO_HASHES_MAX_DEPTH {
        for j in 0..POSEIDON_M31X16_RATE {
            assert_eq!(zero_hashes_poseidon[i][j], ZERO_HASHES_POSEIDON[i][j]);
        }
    }
}
pub fn merkle_tree_element_with_limit(
    leaves: &[Vec<u32>],
    param: &PoseidonParams,
    limit: usize,
) -> Result<Vec<Vec<Vec<u32>>>, String> {
    if leaves.is_empty() {
        return Err("no leaves".to_string());
    }

    let mut length = leaves.len();
    if limit != 0 {
        if length > limit {
            panic!("The length of leaves is larger than the limit");
        }
        length = limit;
    }

    if length == 1 {
        return Ok(vec![leaves.to_vec()]);
    }

    let depth = (length as f64).log2().ceil() as usize + 1;
    let mut tree: Vec<Vec<Vec<u32>>> = vec![vec![]; depth];

    let mut cur_level = leaves.to_vec();
    for i in 0..depth - 1 {
        if cur_level.len() % 2 == 1 {
            cur_level.push(ZERO_HASHES_POSEIDON[i].to_vec()); // Assume this function is defined
        }

        tree[depth - i - 1] = cur_level.clone();

        let mut new_level = Vec::new();
        for j in (0..cur_level.len()).step_by(2) {
            let mut combined = cur_level[j].clone();
            combined.extend_from_slice(&cur_level[j + 1]);
            let new_hash = param.hash(&combined); // Assume this function is defined
            new_level.push(new_hash);
        }

        cur_level = new_level;
    }

    tree[0] = cur_level;
    Ok(tree)
}


pub fn merkleize_var_with_limit<C: Config, B: RootAPI<C>>(
    builder: &mut B,
    leaves: &[Vec<Variable>],
    param: &PoseidonM31Params,
    limit: usize,
) -> Vec<Variable> {
    if leaves.is_empty() {
        panic!("no leaves");
    }

    let mut length = leaves.len();
    if limit != 0 {
        if length > limit {
            panic!("The length of leaves is larger than the limit");
        }
        length = limit;
    }

    if length == 1 {
        return leaves[0].clone();
    }

    let depth = (length as f64).log2().ceil() as usize + 1;

    let mut cur_level = leaves.to_vec();
    for i in 0..depth - 1 {
        if cur_level.len() % 2 == 1 {
            let mut padding = vec![builder.constant(0); param.rate];
            (0..param.rate).for_each(|j| padding[j] = builder.constant(ZERO_HASHES_POSEIDON[i+10][j]));
            cur_level.push(padding); // Assume this function is defined
        }

        let mut new_level = Vec::new();
        for j in (0..cur_level.len()).step_by(2) {
            let mut combined = cur_level[j].clone();
            combined.extend_from_slice(&cur_level[j + 1]);
            let new_hash = param.hash_to_state_flatten(builder, &combined)[..param.rate].to_vec(); // Assume this function is defined
            new_level.push(new_hash);
        }
        cur_level = new_level;
    }
    cur_level[0].clone()
}

pub fn merkleize_with_mixin_poseidon(root: &[u32], num: u64, param: &PoseidonParams) -> Vec<u32> {
    // Convert num into bytes (LittleEndian)
    let mut num_bytes = [0u8; 8];
    num_bytes.as_mut().write_u64::<LittleEndian>(num).unwrap();

    // Convert root to u32 elements
    let mut combined: Vec<u32> = root.to_vec();
    (0..8).for_each(|i| combined.push(num_bytes[i] as u32));
    // Poseidon hash function (assume poseidon_elements_unsafe exists)
    param.hash(&combined)
}

pub fn verify_merkle_tree_path_var<C: Config, B: RootAPI<C>>(
    builder: &mut B,
    root: &[Variable],
    leaf: &[Variable],
    path: &[Variable],
    aunts: &[Vec<Variable>],
    params: &PoseidonM31Params,
    ignore_opt: Variable,
) {
    let depth = path.len();
    let mut cur_leaf = leaf.to_vec();
    let one_var = builder.constant(1);
    for i in 0..depth {
        //if path[i] is -1, set reach_end to 1
        let zero_flag = builder.add(path[i], one_var);
        let reach_end = builder.is_zero(zero_flag);

        //start merging the leaf with the aunts
        let is_left = builder.is_zero(path[i]);
        let mut left = cur_leaf.clone();
        let mut right = cur_leaf.clone();
        for j in 0..cur_leaf.len() {
            left[j] = simple_select(builder, is_left, cur_leaf[j], aunts[i][j]);
            right[j] = simple_select(builder, is_left, aunts[i][j], cur_leaf[j]);
        }
        let mut combined = left.clone();
        combined.extend_from_slice(&right);
        let mut new_leaf;
        if root.len() == 32 {
            //sha256 merkletree
            new_leaf = sha256::m31::sha256_var_bytes(builder, &combined);
        } else if root.len() == params.rate {
            //poseidon merkletree
            new_leaf = params.hash_to_state_flatten(builder, &combined)[..params.rate].to_vec();
        } else {
            panic!("Unsupported type of merkle tree");
        }
        for j in 0..cur_leaf.len() {
            new_leaf[j] = simple_select(builder, reach_end, cur_leaf[j], new_leaf[j]);
        }
        cur_leaf = new_leaf;
    }
    for i in 0..root.len() {
        cur_leaf[i] = simple_select(builder, ignore_opt, root[i], cur_leaf[i]);
        builder.assert_is_equal(cur_leaf[i], root[i]);
    }
}

pub fn calculate_merkle_tree_root_var<C: Config, B: RootAPI<C>>(
    builder: &mut B,
    aunts: &[Vec<Variable>],
    path: &[Variable],
    leaf: Vec<Variable>,
    params: &PoseidonM31Params,
) -> Vec<Variable> {
    let index_bin = path;
    let mut cur_leaf = leaf;
    let one_var = builder.constant(1);
    let zero_var = builder.constant(0);
    for i in 0..path.len() {
        let non_neg_flag = builder.add(index_bin[i], one_var);
        let reach_end = builder.is_zero(non_neg_flag);
        let is_left = builder.is_zero(index_bin[i]);

        let mut left = vec![zero_var; cur_leaf.len()];
        let mut right = vec![zero_var; cur_leaf.len()];

        for j in 0..cur_leaf.len() {
            left[j] = simple_select(builder, is_left, cur_leaf[j], aunts[i][j]);
            right[j] = simple_select(builder, is_left, aunts[i][j], cur_leaf[j]);
        }

        let mut combined = Vec::with_capacity(left.len() + right.len());
        combined.extend_from_slice(&left);
        combined.extend_from_slice(&right);

        let new_leaf;
        if cur_leaf.len() == 32 {
            //sha256 merkletree
            new_leaf = sha256::m31::sha256_var_bytes(builder, &combined);
        } else if cur_leaf.len() == params.rate {
            //poseidon merkletree
            new_leaf = params.hash_to_state_flatten(builder, &combined)[..params.rate].to_vec();
        } else {
            panic!("Unsupported type of merkle tree");
        }
        for j in 0..new_leaf.len() {
            cur_leaf[j] = simple_select(builder, reach_end, cur_leaf[j], new_leaf[j]);
        }
    }

    cur_leaf
}
