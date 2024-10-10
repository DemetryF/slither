use ecolor::Color32;
use emath::Pos2;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PlayerJoin {
    pub color: Option<Color32>,
    pub nickname: String,
}

#[derive(Serialize, Deserialize)]
pub enum ClientUpdate {
    Direction(f32),
    Disconnect,
}

#[derive(Serialize, Deserialize)]
pub struct SessionStart {
    pub world_size: Pos2,
}

#[derive(Serialize, Deserialize)]
pub enum ServerUpdate {
    GameOver,
    World,
    PlayersTop,
}
