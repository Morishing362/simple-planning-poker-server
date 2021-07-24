use super::super::entity;
use super::super::handler;

use warp;

#[derive(Clone)]
pub struct ClientRouter {
	clients: entity::Clients,
	field_cards: entity::FieldCards,
	field_state: entity::FieldState,
}

impl ClientRouter {
	pub fn new(
		clients: entity::Clients,
		field_cards: entity::FieldCards,
		field_state: entity::FieldState,
	) -> ClientRouter {
		ClientRouter {
			clients: clients,
			field_cards: field_cards,
			field_state: field_state,
		}
	}

	pub async fn route(&self, message: warp::ws::Message) {
		let message_str = &message.to_str().unwrap();
		let input: entity::Input<String> = serde_json::from_str(message_str).unwrap();

		let proc_id = input.proc_id.clone();
		let data = input.data.clone();

		if proc_id == String::from("publish") {
			let message_content = entity::MessageContent {
				user_id: data["user_id"].clone(),
				message: data["message"].clone(),
			};
			handler::publish(message_content, self.clients.clone()).await;
		}
		if proc_id == String::from("card_post") {
			let card = entity::Card {
				user_id: data["user_id"].clone(),
				number: data["number"].clone().parse::<usize>().unwrap(),
			};
			handler::card_post(card, self.field_cards.clone()).await;
		}
		if proc_id == String::from("card_delete") {
			let user_id = data["user_id"].clone();
			handler::card_delete(user_id, self.field_cards.clone()).await;
		}
	}
}
