use byteorder::{LittleEndian, WriteBytesExt};
use circuit_std_rs::{
    poseidon::{
        poseidon::PoseidonParams, poseidon_m31::PoseidonM31Params, utils::POSEIDON_M31X16_RATE,
    }, sha256, utils::simple_select
};
use expander_compiler::frontend::{Config, RootAPI, Variable};
pub const MAX_BEACON_VALIDATOR_DEPTH: usize = 40;
const ZERO_HASHES_MAX_DEPTH: usize = MAX_BEACON_VALIDATOR_DEPTH;
const ZERO_HASHES_POSEIDON: [&[u32]; ZERO_HASHES_MAX_DEPTH] = [
    &[0, 0, 0, 0, 0, 0, 0, 0],
    &[
        1479731193, 675523649, 2589942, 996409316, 662065262, 1747716529, 2069769266, 80342673,
    ],
    &[
        675911415, 601570591, 1278688425, 1894163601, 55092623, 1525022421, 1138087967, 543581447,
    ],
    &[
        230733636, 1249644294, 1361151000, 1386720405, 852032236, 499664371, 983666517, 1047739623,
    ],
    &[
        1291659711, 324890902, 1298533425, 335261879, 1238915875, 107337122, 768799352, 1893896877,
    ],
    &[
        1485306931, 1583489121, 709229118, 1560151939, 740639551, 426490281, 996567293, 1491324752,
    ],
    &[
        407604176, 722207430, 1515918052, 1740664993, 935726487, 1348686325, 504245345, 574253254,
    ],
    &[
        474259349, 848751307, 1199587180, 305341344, 44563050, 1376002020, 917960859, 1580822754,
    ],
    &[
        959012967, 206582687, 188721559, 156065621, 727799102, 1681411166, 329807999, 316422402,
    ],
    &[
        1343799254, 2056299156, 1882445379, 491693060, 422661481, 704382026, 1586146251, 1586392298,
    ],
    &[
        1891891509, 1703011903, 456396978, 1823789867, 1895002270, 1544411587, 955482154,
        1519334902,
    ],
    &[
        1719008873, 2033343518, 951068730, 2090280912, 371588516, 1428143370, 1760847107,
        1289628861,
    ],
    &[
        872565520, 2077103366, 828509115, 987285686, 2039682157, 1275725217, 232390149, 1305684344,
    ],
    &[
        230394868, 1883532612, 1895027784, 2097696677, 1804605033, 807216292, 313745570, 485696778,
    ],
    &[
        322652487, 1206697246, 437648491, 976693620, 1556333584, 1980963076, 911549079, 774648661,
    ],
    &[
        906946304, 1933072652, 1439488203, 1378734006, 1224211498, 563228583, 753579020, 1971456239,
    ],
    &[
        489737809, 816268954, 785441566, 684309946, 1054455719, 507495449, 409864477, 1779996096,
    ],
    &[
        836596589, 885054823, 2121168388, 66297819, 679702358, 1215082418, 1142736256, 1160408773,
    ],
    &[
        841722610, 101403396, 1016714509, 1499758581, 455388949, 278072151, 1903159588, 1598328078,
    ],
    &[
        1276423190, 1925408134, 859997499, 2120108207, 1708476877, 807181818, 556249413, 956599737,
    ],
    &[
        1749004026, 1396960394, 1556226519, 929292130, 698637432, 777396037, 2099916367, 1698047559,
    ],
    &[
        1123521692, 1978879199, 1779159268, 288309646, 439572057, 235070764, 1008966474, 1657477546,
    ],
    &[
        2095084067, 1761328698, 1495551561, 1926354975, 809772542, 781155065, 369888645, 89187628,
    ],
    &[
        1081966377, 1956860034, 884202868, 807687963, 12522595, 1517693072, 735600114, 1741412983,
    ],
    &[
        258512991, 1066915711, 2132666708, 1889622940, 2063932771, 1397954086, 1960499216,
        1473341012,
    ],
    &[
        935716901, 1003406983, 764349029, 491808930, 2028475833, 248075841, 1112605954, 674555014,
    ],
    &[
        868196828, 400064847, 173415019, 267580328, 1654252964, 689036182, 2114675694, 11575724,
    ],
    &[
        2003601308, 29419069, 128076405, 252245354, 686579250, 897979018, 1903623116, 1558596277,
    ],
    &[
        503932463, 725265098, 948193881, 987280122, 378195420, 1682313277, 914492157, 1495779902,
    ],
    &[
        1735674699, 1150825025, 2040870097, 137339513, 1842204508, 611719266, 1360936602,
        1006783358,
    ],
    &[
        1751858224, 117014736, 1536354030, 862366589, 1510302141, 246626413, 903182191, 181477597,
    ],
    &[
        363046802, 2042725710, 1658617620, 633255923, 1123259559, 1905342925, 2069568949,
        1645200915,
    ],
    &[
        133338870, 23476666, 656849647, 1121196440, 1816862285, 1761125036, 1998156522, 866507190,
    ],
    &[
        14911216, 1326605162, 1526974059, 264532284, 236022651, 79055144, 1478998306, 1008345151,
    ],
    &[
        124050929, 1748808941, 1929922902, 425147893, 569048525, 1605392660, 1794199739, 1350615314,
    ],
    &[
        1804721907, 625030149, 675653353, 836626359, 670332689, 1347708147, 1021247581, 288918650,
    ],
    &[
        1334673313, 915137680, 367251079, 1616323879, 1108257147, 1885316018, 553386730, 1837045110,
    ],
    &[
        587273736, 1199228966, 742510437, 529628190, 1374092030, 642409966, 1480839937, 1083140005,
    ],
    &[
        964484198, 1058192513, 1852340164, 775962209, 1433499840, 1479155367, 125351316, 936988257,
    ],
    &[
        1349495710, 162720725, 395847181, 1724879576, 1620535833, 288000744, 754665002, 8270279,
    ],
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
        if root.len() == 32 {   //sha256 merkletree
            new_leaf = sha256::m31::sha256_var_bytes(builder, &combined);
        } else if root.len() == params.rate {    //poseidon merkletree
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
        if cur_leaf.len() == 32 {   //sha256 merkletree
            new_leaf = sha256::m31::sha256_var_bytes(builder, &combined);
        } else if cur_leaf.len() == params.rate {    //poseidon merkletree
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