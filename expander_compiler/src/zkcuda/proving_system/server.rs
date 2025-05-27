use axum::{
    routing::{get, post},
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize)]
enum RequestType {
    PCSSetup(usize, usize), // (local_val_len, mpi_world_size)
    RegisterKernel,
    CommitInput,
    Prove,
}

async fn request_handler(Json(payload): Json<RequestType>) -> Json<bool> {
    
    
    axum::Json(true)
}

// #[tokio::main]
// async fn main() {
//     let app = Router::new()
//         .route("/", get(hello))
//         .route("/users", post(create_user));

//     let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//     println!("Server running at http://{}", addr);
//     let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
//     axum::serve(listener, app.into_make_service())
//         .await
//         .unwrap();
// }
