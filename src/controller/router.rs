use core::convert::Infallible;
use warp::{Filter, Rejection, Reply};

use super::super::entity;
use super::super::handler;

pub struct Router {
	clients: entity::Clients,
	field_cards: entity::FieldCards,
}

impl Router {
	pub fn new(clients: entity::Clients, field_cards: entity::FieldCards) -> Router {
		Router {
			clients: clients,
			field_cards: field_cards,
		}
	}

	pub fn all_route(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
		let register_routes = self.register_routes();
		let publish_route = self.publish_route();
		let card_route = self.card_route();
		let ws_route = self.ws_route();
		self.health_route()
			.or(register_routes)
			.or(publish_route)
			.or(card_route)
			.or(ws_route)
	}

	fn health_route(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
		warp::path!("health").and_then(handler::health_handler)
	}

	fn register_routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
		let register_route = warp::path("register");
		register_route
			.and(warp::post())
			.and(warp::body::json())
			.and(with_clients(self.clients.clone()))
			.and_then(handler::register_handler)
			.or(register_route
				.and(warp::delete())
				.and(warp::path::param())
				.and(with_clients(self.clients.clone()))
				.and_then(handler::unregister_handler))
	}

	fn publish_route(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
		warp::path!("publish")
			.and(warp::body::json())
			.and(with_clients(self.clients.clone()))
			.and_then(handler::publish_handler)
	}

	fn card_route(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
		let card_route = warp::path("card");
		card_route
			.and(warp::post())
			.and(warp::body::json())
			.and(with_field_cards(self.field_cards.clone()))
			.and_then(handler::card_post_handler)
			.or(card_route
				.and(warp::delete())
				.and(warp::path::param())
				.and(with_field_cards(self.field_cards.clone()))
				.and_then(handler::card_delete_handler))
	}

	fn ws_route(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
		warp::path("ws")
			.and(warp::ws())
			.and(warp::path::param())
			.and(with_clients(self.clients.clone()))
			.and_then(handler::ws_handler)
	}
}

fn with_clients(
	clients: entity::Clients,
) -> impl Filter<Extract = (entity::Clients,), Error = Infallible> + Clone {
	warp::any().map(move || clients.clone())
}

fn with_field_cards(
	field_cards: entity::FieldCards,
) -> impl Filter<Extract = (entity::FieldCards,), Error = Infallible> + Clone {
	warp::any().map(move || field_cards.clone())
}
