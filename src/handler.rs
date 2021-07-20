use serde::{Deserialize, Serialize};
use serde_json;
use std::convert::Infallible;
use uuid::Uuid;
use warp::{http::StatusCode, reply::json, ws::Message, Reply};

use super::controller::ws;
use super::entity;

#[derive(Serialize, Debug)]
pub struct RegisterResponse {
	pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
	pub user_id: String,
}

pub async fn card_post_handler(
	body: entity::Card,
	field_cards: entity::FieldCards,
) -> Result<impl Reply, Infallible> {
	// WIP: Need to avoid duplicate.
	field_cards.write().await.push(body);
	println!("Here are...");
	field_cards
		.read()
		.await
		.iter()
		.for_each(|card| println!("Card {}", card.number));
	Ok(StatusCode::OK)
}

pub async fn publish_handler(
	body: entity::Content,
	clients: entity::Clients,
) -> Result<impl Reply, Infallible> {
	clients
		.read()
		.await
		.iter()
		.filter(|(_, client)| match &body.user_id {
			Some(v) => true,
			None => false,
		})
		.for_each(|(_, client)| {
			if let Some(sender) = &client.sender {
				let content = entity::Content {
					user_id: body.user_id.clone(),
					message: body.message.clone(),
				};
				let message = serde_json::json!(&content).to_string();
				println!(
					"message from {}: {}",
					body.user_id.clone().unwrap(),
					body.message.clone(),
				);
				let _ = sender.send(Ok(Message::text(message)));
			}
		});

	Ok(StatusCode::OK)
}

pub async fn register_handler(
	body: RegisterRequest,
	clients: entity::Clients,
) -> Result<impl Reply, Infallible> {
	let user_id = body.user_id;
	let uuid = Uuid::new_v4().simple().to_string();

	register_client(uuid.clone(), user_id, clients).await;
	Ok(json(&RegisterResponse {
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
