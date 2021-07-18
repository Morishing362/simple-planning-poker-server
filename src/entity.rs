use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp::ws::Message;

pub type Clients = Arc<RwLock<HashMap<String, Client>>>;

pub type Cards = Arc<RwLock<Vec<Card>>>;

#[derive(Debug, Clone)]
pub struct Client {
	pub user_id: String,
	pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
	pub user_id: Option<String>,
	pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Card {
	pub user_id: Option<String>,
	pub number: usize,
	pub open: bool,
}
