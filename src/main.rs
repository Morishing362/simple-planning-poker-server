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

    let field_cards: entity::FieldCards = Arc::new(RwLock::new(Vec::new()));

    let router = Router::new(clients.clone(), field_cards.clone());

    let routes = router.all_route();

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
