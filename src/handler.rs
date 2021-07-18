use std::convert::Infallible;
use uuid::Uuid;
use warp::{http::StatusCode, reply::json, ws::Message, Reply};

use super::controller::ws;
use super::entity;

pub async fn publish_handler(
	body: entity::Content,
	clients: entity::Clients,
) -> Result<impl Reply, Infallible> {
	clients
		.read()
		.await
		.iter()
		.filter(|(_, client)| match &body.user_id {
			Some(v) => &client.user_id == v,
			None => true,
		})
		.filter(|(_, client)| client.topics.contains(&body.topic))
		.for_each(|(_, client)| {
			if let Some(sender) = &client.sender {
				let _ = sender.send(Ok(Message::text(body.message.clone())));
			}
		});

	Ok(StatusCode::OK)
}

pub async fn register_handler(
	body: entity::RegisterRequest,
	clients: entity::Clients,
) -> Result<impl Reply, Infallible> {
	let user_id = body.user_id;
	let uuid = Uuid::new_v4().simple().to_string();

	register_client(uuid.clone(), user_id, clients).await;
	Ok(json(&entity::RegisterResponse {
		url: format!("ws://127.0.0.1:8000/ws/{}", uuid),
	}))
}

async fn register_client(id: String, user_id: String, clients: entity::Clients) {
	let already_registered = clients
		.read()
		.await
		.iter()
		.any(|(_, client)| client.user_id == user_id);

	if already_registered {
		println!("user_id is already registered.");
	} else {
		clients.write().await.insert(
			id,
			entity::Client {
				user_id,
				topics: vec![String::from("cats")],
				sender: None,
			},
		);
	}
}

pub async fn unregister_handler(
	id: String,
	clients: entity::Clients,
) -> Result<impl Reply, Infallible> {
	clients.write().await.remove(&id);
	Ok(StatusCode::OK)
}

pub async fn ws_handler(
	ws: warp::ws::Ws,
	id: String,
	clients: entity::Clients,
) -> Result<impl Reply, warp::Rejection> {
	let client = clients.read().await.get(&id).cloned();
	match client {
		Some(c) => Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, id, clients, c))),
		None => Err(warp::reject::not_found()),
	}
}

pub async fn health_handler() -> Result<impl Reply, Infallible> {
	Ok(StatusCode::OK)
}
