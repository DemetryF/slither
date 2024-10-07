use core::SlitherID;

use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::sync::mpsc;

use crate::connection::Connection;
use crate::state_updater::ConnectionMessage;

pub struct Listener {
    listener: TcpListener,
    connections_tx: mpsc::Sender<ConnectionMessage>,
    directions_tx: mpsc::Sender<(SlitherID, f32)>,
}

impl Listener {
    pub async fn start_on(
        addr: impl ToSocketAddrs,
        connections_tx: mpsc::Sender<ConnectionMessage>,
        directions_tx: mpsc::Sender<(SlitherID, f32)>,
    ) -> Self {
        let listener = TcpListener::bind(addr).await.unwrap();

        Self {
            listener,
            connections_tx,
            directions_tx,
        }
    }

    pub async fn listen(self) {
        let mut ids_counter = 0;

        let mut next_id = || {
            ids_counter += 1;
            SlitherID(ids_counter - 1)
        };

        loop {
            let (stream, _) = self.listener.accept().await.unwrap();

            let (read_socket, write_socket) = stream.into_split();

            let id = next_id();

            self.connections_tx
                .send(ConnectionMessage::Connected(id, write_socket))
                .await
                .unwrap();

            let connection = Connection {
                id,
                read_socket,
                directions_tx: self.directions_tx.clone(),
                connections_tx: self.connections_tx.clone(),
            };

            tokio::spawn(connection.start());
        }
    }
}
