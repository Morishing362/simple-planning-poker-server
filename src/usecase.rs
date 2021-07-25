use serde::Serialize;
use serde_json;
use warp::ws::Message;

use super::entity;

#[derive(Clone)]
pub struct Usecase {
	clients: entity::Clients,
	field_cards: entity::FieldCards,
	field_state: entity::FieldState,
}

impl Usecase {
	pub fn new(
		clients: entity::Clients,
		field_cards: entity::FieldCards,
		field_state: entity::FieldState,
	) -> Usecase {
		Usecase {
			clients: clients,
			field_cards: field_cards,
			field_state: field_state,
		}
	}

	pub async fn publish(&self, body: entity::MessageContent) {
		let data = entity::MessageContent {
			user_id: body.user_id.clone(),
			message: body.message.clone(),
		};
		let message = self.output_message(String::from("published"), data);
		let _ = self.send_to_all(message).await;
	}

	pub async fn card_post(&self, body: entity::Card) {
		let already_posted = self
			.field_cards
			.read()
			.await
			.iter()
			.any(|card| body.user_id == card.user_id);

		if !already_posted {
			self.field_cards.write().await.push(body);
			let mut cards: Vec<entity::Card> = vec![];
			self.field_cards.read().await.iter().for_each(|card| {
				cards.push(card.clone());
			});
			let message = self.output_vec_message(String::from("card_posted"), cards);
			self.send_to_all(message).await;
			self.print_field_cards().await;
		}
	}

	pub async fn card_delete(&self, user_id: String) {
		let index = self
			.field_cards
			.read()
			.await
			.iter()
			.position(|card| &card.user_id == &user_id);

		if let Some(i) = index {
			self.field_cards.write().await.remove(i);
			self.print_field_cards().await;
			let mut cards: Vec<entity::Card> = vec![];
			self.field_cards.read().await.iter().for_each(|card| {
				cards.push(card.clone());
			});
			let message = self.output_vec_message(String::from("card_deleted"), cards);
			self.send_to_all(message).await;
			self.print_field_cards().await;
		}
	}

	pub async fn result(&self) {
		if self.field_state.read().await.is_open {
			println!("Cards are already open.");
		} else {
			let mut sum: f64 = 0.0;
			let mut size: f64 = 0.0;
			self.field_cards.read().await.iter().for_each(|card| {
				sum += card.number as f64;
				size += 1.0;
			});
			if size == 0.0 {
				println!("error result computation");
			} else {
				let mut new_state = self.field_state.write().await;
				let data = entity::TableState {
					result: sum / size,
					is_open: true,
				};
				*new_state = data.clone();
				let message = self.output_message(String::from("result"), data.clone());
				self.send_to_all(message).await;
				println!("Table state: {:?}", data);
			}
		}
	}

	pub async fn clean(&self) {
		if !self.field_state.read().await.is_open {
			println!("Cards are already cleared.");
		} else {
			self.field_cards.write().await.clear();
			let mut new_state = self.field_state.write().await;
			let data = entity::TableState {
				result: 0.0,
				is_open: false,
			};
			*new_state = data.clone();
			let message = self.output_message(String::from("cleaned"), data.clone());
			self.send_to_all(message).await;
			println!("Table state: {:?}", data);
		}
	}

	fn output_message<T: Serialize>(&self, proc_id: String, data: T) -> String {
		let output_vec = entity::Output::<T> {
			proc_id: String::from(proc_id),
			data: data,
		};
		serde_json::json!(output_vec).to_string()
	}
	fn output_vec_message<T: Serialize>(&self, proc_id: String, vector: Vec<T>) -> String {
		let output_vec = entity::OutputVec::<T> {
			proc_id: String::from(proc_id),
			data: vector,
		};
		serde_json::json!(output_vec).to_string()
	}

	async fn send_to_all(&self, message: String) {
		self.clients.read().await.iter().for_each(|(_, client)| {
			if let Some(sender) = &client.sender {
				let _ = sender.send(Ok(Message::text(&message)));
			}
		});
	}

	async fn print_field_cards(&self) {
		println!("Here are...");
		self.field_cards
			.read()
			.await
			.iter()
			.for_each(|card| println!("Card {}", &card.number));
	}
}
