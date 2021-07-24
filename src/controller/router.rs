use core::convert::Infallible;
use warp::{Filter, Rejection, Reply};

use super::super::controller::client_router;
use super::super::entity;
use super::super::handler;

pub struct Router {
	clients: entity::Clients,
	field_cards: entity::FieldCards,
	field_state: entity::FieldState,
	client_router: client_router::ClientRouter,
}

impl Router {
	pub fn new(
		clients: entity::Clients,
		field_cards: entity::FieldCards,
		field_state: entity::FieldState,
	) -> Router {
		Router {
			clients: clients.clone(),
			field_cards: field_cards.clone(),
			field_state: field_state.clone(),
			client_router: client_router::ClientRouter::new(
				clients.clone(),
				field_cards.clone(),
				field_state.clone(),
			),
		}
	}

	pub fn all_route(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
		let register_routes = self.register_routes();
		let admin_route = self.admin_route();
		let ws_route = self.ws_route();
		self.health_route()
			.or(register_routes)
			.or(admin_route)
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

	fn admin_route(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
		let admin_route = warp::path("admin").and(warp::get());
		admin_route
			.and(warp::path("result"))
			.and(with_field_cards(self.field_cards.clone()))
			.and(with_field_state(self.field_state.clone()))
			.and_then(handler::open_result_handler)
			.or(admin_route
				.and(warp::path("clean"))
				.and(with_field_cards(self.field_cards.clone()))
				.and(with_field_state(self.field_state.clone()))
				.and_then(handler::clean_cards_handler))
	}

	fn ws_route(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
		warp::path("ws")
			.and(warp::ws())
			.and(warp::path::param())
			.and(with_clients(self.clients.clone()))
			.and(with_client_router(self.client_router.clone()))
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

fn with_field_state(
	field_state: entity::FieldState,
) -> impl Filter<Extract = (entity::FieldState,), Error = Infallible> + Clone {
	warp::any().map(move || field_state.clone())
}
fn with_client_router(
	client_router: client_router::ClientRouter,
) -> impl Filter<Extract = (client_router::ClientRouter,), Error = Infallible> + Clone {
	warp::any().map(move || client_router.clone())
}
