use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use warp::ws::WebSocket;

use super::super::entity;

pub async fn client_connection(
	ws: WebSocket,
	id: String,
	clients: entity::Clients,
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

	while let Some(result) = client_ws_rcv.next().await {
		let msg = match result {
			Ok(msg) => msg,
			Err(e) => {
				eprintln!("error receiving ws message for id: {}): {}", id.clone(), e);
				break;
			}
		};
		println!("received message from {}: {:?}", id, msg);
	}

	clients.write().await.remove(&id);
	println!("{} disconnected", id);
}
