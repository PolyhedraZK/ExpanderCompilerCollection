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

pub async fn request_prove(client: &Client, server_url: &str, parallel_count: usize, kernel_id: usize) {
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

pub async fn request_verify(client: &Client, server_url: &str, parallel_count: usize, kernel_id: usize) -> bool {
    let request = RequestType::Verify(parallel_count, kernel_id);
    let res = client
        .post(server_url)
        .json(&request)
        .send()
        .await
        .expect("Failed to send verify request");

    if res.status().is_success() {
        // Assuming the response body contains a boolean indicating success
        match res.json::<bool>().await {
            Ok(success) => success,
            Err(e) => {
                eprintln!("Failed to parse verify response: {}", e);
                false
            }
        }
    } else {
        eprintln!("Verify request failed: {}", res.status());
        false
    }
}

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let client = Client::new();

//     // GET request
//     let res = client
//         .get("http://127.0.0.1:3000/")
//         .send()
//         .await?
//         .text()
//         .await?;

//     println!("GET response: {}", res);

//     let test_size = 10;
//     for i in 0..test_size {
//         let request = if i % 2 == 0 {
//             Request::ADD
//         } else {
//             Request::MUL
//         };

//         let res = client
//             .post("http://127.0.0.1:3000/compute")
//             .json(&request)
//             .send()
//             .await?;

//         let user_response: UserResponse = res.json().await?;
//         println!("POST response: {}", user_response.v);
//     }
//     // Exit request
//     let exit_request = Request::EXIT;
//     let res = client
//             .post("http://127.0.0.1:3000/compute")
//             .json(&exit_request)
//             .send()
//             .await?;
//     let user_response: UserResponse = res.json().await?;
//     println!("Exit response: {}", user_response.v);
//     Ok(())
// }
