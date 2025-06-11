use super::server_utils::{RequestType, get_service_args};

use reqwest::Client;

pub async fn request_setup(setup_file: &str) {
    post_request(RequestType::Setup(setup_file.to_string())).await;
}

pub async fn request_prove() {
    post_request(RequestType::Prove).await;
}

pub async fn request_exit() {
    post_request(RequestType::Exit).await;
}

pub async fn post_request(request: RequestType) {
    let client = Client::new();
    let (ip, port) = get_service_args();
    let server_url = format!("http://{ip}:{port}/");

    let res = client
        .post(server_url)
        .json(&request)
        .send()
        .await
        .expect("Failed to send request");

    if res.status().is_success() {
        println!("Request successful");
    } else {
        eprintln!("Request failed: {}", res.status());
    }
}
