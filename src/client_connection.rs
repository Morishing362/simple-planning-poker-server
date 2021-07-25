use futures::{FutureExt, StreamExt};
use serde::Serialize;
use tokio::sync::mpsc;
use warp::ws::Message;
use warp::ws::WebSocket;

use super::controller::client_router;
use super::entity;

pub async fn client_connection(
	ws: WebSocket,
	id: String,
	clients: entity::Clients,
	client_router: client_router::ClientRouter,
	mut client: entity::Client,
) {
	let (client_ws_sender, mut client_ws_rcv) = ws.split();
	let (client_sender, client_rcv) = mpsc::unbounded_channel();

	tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
		if let Err(e) = result {
			eprintln!("error sending websocket msg: {}", e);
		}
	}));

	client.sender = Some(client_sender);
	clients.write().await.insert(id.clone(), client);
	println!("{} connected", id);

	let mut users: Vec<String> = vec![];
	clients.read().await.iter().for_each(|(_, c)| {
		users.push(c.user_id.clone());
	});
	let message = output_vec_message(String::from("users_presence"), users);
	send_to_all(message, clients.clone()).await;

	while let Some(result) = client_ws_rcv.next().await {
		let msg = match result {
			Ok(m) => m,
			Err(e) => {
				eprintln!("error receiving ws message for id: {}): {}", id.clone(), e);
				break;
			}
		};
		println!("received message from {}: {:?}", id, &msg);
		client_router.route(msg).await;
	}

	clients.write().await.remove(&id);

	let mut users: Vec<String> = vec![];
	clients.read().await.iter().for_each(|(_, c)| {
		users.push(c.user_id.clone());
	});
	let message = output_vec_message(String::from("users_presence"), users);
	send_to_all(message, clients.clone()).await;

	println!("{} disconnected", id);
}

fn output_vec_message<T: Serialize>(proc_id: String, vector: Vec<T>) -> String {
	let output_vec = entity::OutputVec::<T> {
		proc_id: String::from(proc_id),
		data: vector,
	};
	serde_json::json!(output_vec).to_string()
}

async fn send_to_all(message: String, clients: entity::Clients) {
	clients.read().await.iter().for_each(|(_, client)| {
		if let Some(sender) = &client.sender {
			let _ = sender.send(Ok(Message::text(&message)));
		}
	});
}
