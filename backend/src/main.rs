mod connection;
mod listener;
mod state_updater;
mod transfer;

use std::env;
use std::net::{Ipv4Addr, SocketAddr};
use std::process::exit;

use core::{GameState, World};

use listener::Listener;
use state_updater::StateUpdater;
use tokio::sync::{broadcast, mpsc};

#[tokio::main]
async fn main() {
    let port = port();

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

    let ip = Ipv4Addr::new(0, 0, 0, 0);
    let addr = SocketAddr::new(ip.into(), port);

    println!("start on {addr}");

    let listener = tokio::spawn(
        Listener::start_on(addr, connections_tx, directions_tx, crash_rx)
            .await
            .listen(),
    );

    let _ = tokio::join!(updater, listener);
}

fn port() -> u16 {
    let mut args = env::args();

    while let Some(arg) = args.next() {
        if arg == "--port" {
            let Some(port) = args.next() else {
                eprintln!("you must specify a port after \"--port\"");
                exit(1);
            };

            let Ok(port) = u16::from_str_radix(&port, 10) else {
                eprintln!("invalid port: \"{}\"", &port);
                exit(1);
            };

            return port;
        }
    }

    return 0;
}
