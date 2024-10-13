mod connection;
mod listener;
mod state_updater;
mod transfer;

use listener::Listener;
use state_updater::StateUpdater;
use tokio::sync::{broadcast, mpsc};

use core::{GameState, World};

#[tokio::main]
async fn main() {
    let (connections_tx, connections_rx) = mpsc::channel(1);
    let (directions_tx, directions_rx) = mpsc::channel(16);
    let (crash_tx, crash_rx) = broadcast::channel(16);

    let updater = tokio::spawn(
        StateUpdater::new(
            GameState::new(World::new(2000., 2000., 2000.)),
            connections_rx,
            directions_rx,
            crash_tx,
        )
        .start(),
    );

    let listener = tokio::spawn(
        Listener::start_on("192.168.0.11:1488", connections_tx, directions_tx, crash_rx)
            .await
            .listen(),
    );

    let _ = tokio::join!(updater, listener);
}
