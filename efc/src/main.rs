use std::env;

use efc::end2end::end2end_witnesses_streamline_from_beacon_data;

fn main() {
    let args: Vec<String> = env::args().collect();
    end2end_witnesses_streamline_from_beacon_data(290000, "end");
    // if let (Some(s_index), Some(e_index)) = (
    //     args.iter().position(|x| x == "-s"),
    //     args.iter().position(|x| x == "-e"),
    // ) {
    //     match (args.get(s_index + 1), args.get(e_index + 1)) {
    //         (Some(stage), Some(epoch_str)) => match epoch_str.parse::<u64>() {
    //             Ok(epoch) => {
    //                 println!("Running stage: {} at epoch: {}", stage, epoch);
    //                 end2end_witnesses_streamline_from_beacon_data(epoch, stage);
    //             }
    //             Err(_) => println!("Epoch must be a valid number"),
    //         },
    //         _ => println!("Missing value for -s or -e"),
    //     }
    // } else {
    //     println!("Missing -s or -e argument");
    // }
}
