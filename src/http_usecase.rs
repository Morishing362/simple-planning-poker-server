use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use uuid::Uuid;
use warp::{http::StatusCode, reply::json, Rejection, Reply};

use super::client_connection;
use super::controller::client_router;
use super::entity;

#[derive(Serialize, Debug)]
pub struct RegisterResponse {
	pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
	pub user_id: String,
}

pub async fn register(
	body: RegisterRequest,
	clients: entity::Clients,
) -> Result<impl Reply, Rejection> {
	let user_id = body.user_id;
	let uuid = Uuid::new_v4().simple().to_string();

	match register_client(uuid.clone(), user_id, clients).await {
		Ok(_) => Ok(json(&RegisterResponse {
			url: format!("ws://127.0.0.1:8000/ws/{}", uuid),
		})),
		Err(_) => Err(warp::reject::reject()),
	}
}

async fn register_client(
	id: String,
	user_id: String,
	clients: entity::Clients,
) -> Result<bool, bool> {
	let already_registered = clients
		.read()
		.await
		.iter()
		.any(|(_, client)| client.user_id == user_id);

	if already_registered {
		println!("user_id is already registered.");
		Err(false)
	} else {
		clients.write().await.insert(
			id,
			entity::Client {
				user_id,
				sender: None,
			},
		);
		Ok(true)
	}
}

pub async fn unregister(id: String, clients: entity::Clients) -> Result<impl Reply, Infallible> {
	clients.write().await.remove(&id);
	Ok(StatusCode::OK)
}

pub async fn ws_handler(
	ws: warp::ws::Ws,
	id: String,
	clients: entity::Clients,
	client_router: client_router::ClientRouter,
) -> Result<impl Reply, warp::Rejection> {
	let client = clients.read().await.get(&id).cloned();
	match client {
		Some(c) => Ok(ws.on_upgrade(move |socket| {
			client_connection::client_connection(socket, id, clients, client_router, c)
		})),
		None => Err(warp::reject::not_found()),
	}
}

pub async fn health_usecase() -> Result<impl Reply, Infallible> {
	Ok(StatusCode::OK)
}
