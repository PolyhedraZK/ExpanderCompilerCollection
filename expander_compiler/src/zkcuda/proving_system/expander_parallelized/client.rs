use super::vanilla_utils::{RequestType, SERVER_IP, SERVER_PORT};

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
    let port = {
        let port = SERVER_PORT.lock().unwrap();
        *port
    };
    let server_url = format!("{SERVER_IP}:{port}");
    let server_url = format!("http://{server_url}/");

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
