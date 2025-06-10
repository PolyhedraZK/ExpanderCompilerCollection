use super::server_utils::{RequestType, SERVER_IP, SERVER_PORT};

use reqwest::Client;

pub async fn request_setup(setup_file: &str) {
    println!("Sending Setup request to server");
    post_request(RequestType::Setup(setup_file.to_string())).await;
}

pub async fn request_prove() {
    post_request(RequestType::Prove).await;
}

pub async fn request_exit() {
    post_request(RequestType::Exit).await;
}

pub async fn post_request(request: RequestType) {
    println!("post_request");
    let client = Client::new();
    let port = SERVER_PORT.lock().unwrap();
    let server_url = format!("{}:{}", SERVER_IP, *port);
    drop(port);
    let server_url = format!("http://{}/", server_url);
    println!("Sending request {:?} to server at {}", request, server_url);

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
