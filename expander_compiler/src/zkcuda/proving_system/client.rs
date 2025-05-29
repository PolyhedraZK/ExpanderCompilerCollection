use super::server::RequestType;

use reqwest::Client;

pub async fn request_commit_input(client: &Client, server_url: &str, parallel_count: usize) {
    let request = RequestType::CommitInput(parallel_count);
    let res = client
        .post(server_url)
        .json(&request)
        .send()
        .await
        .expect("Failed to send input commitment request");

    if res.status().is_success() {
        println!("Input commitment request successful");
    } else {
        eprintln!("Input commitment request failed: {}", res.status());
    }
}

pub async fn request_prove(
    client: &Client,
    server_url: &str,
    parallel_count: usize,
    kernel_id: usize,
) {
    let request = RequestType::Prove(parallel_count, kernel_id);
    let res = client
        .post(server_url)
        .json(&request)
        .send()
        .await
        .expect("Failed to send prove request");

    if res.status().is_success() {
        println!("Prove request successful");
    } else {
        eprintln!("Prove request failed: {}", res.status());
    }
}

// pub async fn request_verify(
//     client: &Client,
//     server_url: &str,
//     parallel_count: usize,
//     kernel_id: usize,
// ) -> bool {
//     let request = RequestType::Verify(parallel_count, kernel_id);
//     let res = client
//         .post(server_url)
//         .json(&request)
//         .send()
//         .await
//         .expect("Failed to send verify request");

//     if res.status().is_success() {
//         // Assuming the response body contains a boolean indicating success
//         match res.json::<bool>().await {
//             Ok(success) => success,
//             Err(e) => {
//                 eprintln!("Failed to parse verify response: {}", e);
//                 false
//             }
//         }
//     } else {
//         eprintln!("Verify request failed: {}", res.status());
//         false
//     }
// }
