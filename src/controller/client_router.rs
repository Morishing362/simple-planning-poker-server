use super::super::entity;
use super::super::usecase;

use warp;

#[derive(Clone)]
pub struct ClientRouter {
	usecase: usecase::Usecase,
}

impl ClientRouter {
	pub fn new(usecase: usecase::Usecase) -> ClientRouter {
		ClientRouter { usecase: usecase }
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
			self.usecase.publish(message_content).await;
		}
		if proc_id == String::from("card_post") {
			let card = entity::Card {
				user_id: data["user_id"].clone(),
				number: data["number"].clone().parse::<usize>().unwrap(),
			};
			self.usecase.card_post(card).await;
		}
		if proc_id == String::from("card_delete") {
			let user_id = data["user_id"].clone();
			self.usecase.card_delete(user_id).await;
		}
		if proc_id == String::from("result") {
			self.usecase.result().await;
		}
		if proc_id == String::from("clean") {
			self.usecase.clean().await;
		}
	}
}
