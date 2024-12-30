

pub fn nb_multiplication_res_limbs(len_left: usize, len_right: usize) -> usize {
    let res = len_left + len_right - 1;
    if res < 0 {
        0
    } else {
        res
    }
}