use core::convert::Infallible;
use warp::{Filter, Reply};

use super::super::entity;
use super::super::handler;

pub struct Router {
	clients: entity::Clients,
}

impl Router {
	pub fn new(clients: entity::Clients) -> Router {
		Router { clients: clients }
	}

	pub fn all_route(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
		let register_routes = self.register_routes();
		let publish_route = self.publish_route();
		let ws_route = self.ws_route();
		self.health_route()
			.or(register_routes)
			.or(publish_route)
			.or(ws_route)
	}

	fn health_route(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
		warp::path!("health").and_then(handler::health_handler)
	}

	fn register_routes(
		&self,
	) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
		warp::path("register")
			.and(warp::post())
			.and(warp::body::json())
			.and(with_clients(self.clients.clone()))
			.and_then(handler::register_handler)
			.or((warp::delete())
				.and(warp::path::param())
				.and(with_clients(self.clients.clone()))
				.and_then(handler::unregister_handler))
	}

	fn publish_route(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
		warp::path!("publish")
			.and(warp::body::json())
			.and(with_clients(self.clients.clone()))
			.and_then(handler::publish_handler)
	}

	fn ws_route(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
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
