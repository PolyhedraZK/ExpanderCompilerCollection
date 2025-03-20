use std::env;

use efc::end2end::end2end_witness;
use efc::hashtable::test_hashtable;
use efc::zkcuda_hashtable::test_zkcuda_hashtable;

fn main() {
    test_hashtable();
    //test_zkcuda_hashtable();
    return;

    let args: Vec<String> = env::args().collect();

    // 查找 `-f` 参数的值
    if let Some(f_index) = args.iter().position(|x| x == "-d") {
        if let Some(dir) = args.get(f_index + 1) {
            println!("The directory of -d is: {}", dir);
            end2end_witness(dir);
        } else {
            println!("Directory is not specified, default dir is the current directory");
            end2end_witness(".");
        }
    } else {
        println!("Directory is not specified, default dir is the current directory");
        end2end_witness(".");
    }
}

