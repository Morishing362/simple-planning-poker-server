use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp;

mod controller;
mod entity;
mod handler;

use controller::router::Router;

#[tokio::main]
async fn main() {
    let clients: entity::Clients = Arc::new(RwLock::new(HashMap::new()));

    let server_controller = Router::new(clients.clone());

    let routes = server_controller.all_route();

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
