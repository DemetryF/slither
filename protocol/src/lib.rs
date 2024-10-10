use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ClientUpdate {
    Direction(f32),
    Disconnect,
}
