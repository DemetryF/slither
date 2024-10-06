mod state_updater;

use state_updater::StateUpdater;
use tokio::sync::mpsc;

use core::{GameState, World};

#[tokio::main]
async fn main() {
    let (connections_tx, connections_rx) = mpsc::channel(1);
    let (directions_tx, directions_rx) = mpsc::channel(16);

    tokio::spawn(
        StateUpdater::new(
            GameState::new(World::new(2000., 2000., 2000.)),
            connections_rx,
            directions_rx,
        )
        .start(),
    );
}
