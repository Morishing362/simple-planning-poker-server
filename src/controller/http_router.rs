use core::convert::Infallible;
use warp::{Filter, Rejection, Reply};

use super::super::controller::client_router;
use super::super::entity;
use super::super::http_usecase;

pub fn all_route(
	clients: entity::Clients,
	c_router: client_router::ClientRouter,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
	let cors = warp::cors()
		.allow_any_origin()
		.allow_headers(vec![
			"Sec-WebSocket-Accept",
			"User-Agent",
			"Access-Control-Allow-Origin",
			"Access-Control-Request-Headers",
			"Content-Type",
		])
		.allow_methods(vec!["GET", "POST", "DELETE"]);
	let health = health_route();
	let register = register_route(clients.clone());
	let ws = ws_route(clients.clone(), c_router.clone());
	health.or(register).or(ws).with(cors.clone())
}

pub fn health_route() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
	warp::path!("health").and_then(http_usecase::health_usecase)
}

fn register_route(
	clients: entity::Clients,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
	let register_route = warp::path("register");
	register_route
		.and(warp::post())
		.and(warp::body::json())
		.and(with_clients(clients.clone()))
		.and_then(http_usecase::register)
		.or(register_route
			.and(warp::delete())
			.and(warp::path::param())
			.and(with_clients(clients.clone()))
			.and_then(http_usecase::unregister))
}

fn ws_route(
	clients: entity::Clients,
	client_router: client_router::ClientRouter,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
	warp::path("ws")
		.and(warp::ws())
		.and(warp::path::param())
		.and(with_clients(clients.clone()))
		.and(with_client_router(client_router.clone()))
		.and_then(http_usecase::ws_handler)
}

fn with_clients(
	clients: entity::Clients,
) -> impl Filter<Extract = (entity::Clients,), Error = Infallible> + Clone {
	warp::any().map(move || clients.clone())
}

fn with_client_router(
	client_router: client_router::ClientRouter,
) -> impl Filter<Extract = (client_router::ClientRouter,), Error = Infallible> + Clone {
	warp::any().map(move || client_router.clone())
}
