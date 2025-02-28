use std::env;

// use efc::end2end::end2end_witness_streamline;

// fn main() {
//     let args: Vec<String> = env::args().collect();

//     if let Some(f_index) = args.iter().position(|x| x == "-d") {
//         if let Some(dir) = args.get(f_index + 1) {
//             println!("The directory of -d is: {}", dir);
//             end2end_witness(dir);
//         } else {
//             println!("Directory is not specified, default dir is the current directory");
//             end2end_witness(".");
//         }
//     } else {
//         println!("Directory is not specified, default dir is the current directory");
//         end2end_witness(".");
//     }
// }

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some(f_index) = args.iter().position(|x| x == "-s") {
        if let Some(stage) = args.get(f_index + 1) {
            println!("The stage of -s is: {}", stage);
            // stacker::grow(32 * 1024 * 1024 * 1024, || {
            //     end2end_witness_streamline(stage);
            // });
        } else {
            println!("stage must be specified");
        }
    } else {
        println!("Stage must be specified");
    }
}
