use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp;

mod client_connection;
mod controller;
mod entity;
mod http_usecase;
mod usecase;

#[tokio::main]
async fn main() {
    let clients: entity::Clients = Arc::new(RwLock::new(HashMap::new()));
    let field_cards: entity::FieldCards = Arc::new(RwLock::new(Vec::new()));
    let field_state: entity::FieldState = Arc::new(RwLock::new(entity::TableState {
        result: 0.0,
        is_open: false,
    }));
    let usecase = usecase::Usecase::new(clients.clone(), field_cards.clone(), field_state.clone());
    let c_router = controller::client_router::ClientRouter::new(usecase);
    let routes = controller::http_router::all_route(clients.clone(), c_router.clone());

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
