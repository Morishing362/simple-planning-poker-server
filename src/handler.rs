use serde::{Deserialize, Serialize};
use serde_json;
use std::convert::Infallible;
use uuid::Uuid;
use warp::{http::StatusCode, reply::json, ws::Message, Rejection, Reply};

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

pub async fn publish(body: entity::MessageContent, clients: entity::Clients) {
	clients.read().await.iter().for_each(|(_, client)| {
		if let Some(sender) = &client.sender {
			let content = entity::MessageContent {
				user_id: body.user_id.clone(),
				message: body.message.clone(),
			};
			let message = serde_json::json!(&content).to_string();
			println!(
				"message from {}: {}",
				body.user_id.clone(),
				body.message.clone(),
			);
			let _ = sender.send(Ok(Message::text(message)));
		}
	});
}

pub async fn card_post(body: entity::Card, field_cards: entity::FieldCards) {
	let already_posted = field_cards
		.read()
		.await
		.iter()
		.any(|card| body.user_id == card.user_id);

	if !already_posted {
		field_cards.write().await.push(body);
		print_field_cards(field_cards.clone()).await;
	}
}

pub async fn card_delete(user_id: String, field_cards: entity::FieldCards) {
	let index = field_cards
		.read()
		.await
		.iter()
		.position(|card| &card.user_id == &user_id);

	if let Some(i) = index {
		field_cards.write().await.remove(i);
		print_field_cards(field_cards.clone()).await;
	}
}

pub async fn open_result_handler(
	field_cards: entity::FieldCards,
	field_state: entity::FieldState,
) -> Result<impl Reply, Infallible> {
	if field_state.read().await.is_open {
		println!("Cards are already open.");
	} else {
		let mut sum: f64 = 0.0;
		let mut size: f64 = 0.0;
		field_cards.read().await.iter().for_each(|card| {
			sum += card.number as f64;
			size += 1.0;
		});
		let mut new_state = field_state.write().await;
		*new_state = entity::TableState {
			result: sum / size,
			is_open: true,
		};
		println!("Cards are open.");
	}
	Ok(StatusCode::OK)
}

pub async fn clean_cards_handler(
	field_cards: entity::FieldCards,
	field_state: entity::FieldState,
) -> Result<impl Reply, Infallible> {
	if !field_state.read().await.is_open {
		println!("Cards are already cleared.");
	} else {
		field_cards.write().await.clear();
		let mut new_state = field_state.write().await;
		*new_state = entity::TableState {
			result: 0.0,
			is_open: false,
		};
		print_field_cards(field_cards.clone()).await;
		println!("Cards are cleared.");
	}
	Ok(StatusCode::OK)
}

pub async fn register_handler(
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

pub async fn health_handler() -> Result<impl Reply, Infallible> {
	Ok(StatusCode::OK)
}

async fn print_field_cards(field_cards: entity::FieldCards) {
	println!("Here are...");
	field_cards
		.read()
		.await
		.iter()
		.for_each(|card| println!("Card {}", &card.number));
}
