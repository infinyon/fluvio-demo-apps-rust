use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BotMsg {
    BotText(String),
    UserText(String),
    ChoiceRequest(Vec<Item>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub next: usize,
    pub answer: String,
}
