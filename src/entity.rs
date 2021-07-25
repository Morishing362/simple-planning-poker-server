use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp::ws::Message;

pub type Clients = Arc<RwLock<HashMap<String, Client>>>;

pub type FieldCards = Arc<RwLock<Vec<Card>>>;

pub type FieldState = Arc<RwLock<TableState>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Input<T> {
	pub proc_id: String,
	pub data: HashMap<String, T>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output<T: Serialize> {
	pub proc_id: String,
	pub data: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutputVec<T: Serialize> {
	pub proc_id: String,
	pub data: Vec<T>,
}

#[derive(Debug, Clone)]
pub struct Client {
	pub user_id: String,
	pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageContent {
	pub user_id: String,
	pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Card {
	pub user_id: String,
	pub number: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TableState {
	pub result: f64,
	pub is_open: bool,
}
