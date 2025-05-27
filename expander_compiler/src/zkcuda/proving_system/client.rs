use super::server::RequestType;

use reqwest::Client;

pub async fn request_pcs_setup(
    client: &Client,
    server_url: &str,
    local_val_len: usize,
    mpi_world_size: usize,
) {
    let request = RequestType::PCSSetup(local_val_len, mpi_world_size);
    let res = client
        .post(server_url)
        .json(&request)
        .send()
        .await
        .expect("Failed to send PCS setup request");

    if res.status().is_success() {
        println!("PCS setup request successful");
    } else {
        eprintln!("PCS setup request failed: {}", res.status());
    }
}

pub async fn request_register_kernel(client: &Client, server_url: &str) {
    let request = RequestType::RegisterKernel;
    let res = client
        .post(server_url)
        .json(&request)
        .send()
        .await
        .expect("Failed to send kernel registration request");

    if res.status().is_success() {
        println!("Kernel registration request successful");
    } else {
        eprintln!("Kernel registration request failed: {}", res.status());
    }
}

pub async fn request_commit_input(client: &Client, server_url: &str) {
    let request = RequestType::CommitInput;
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

pub async fn request_prove(client: &Client, server_url: &str) {
    let request = RequestType::Prove;
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
