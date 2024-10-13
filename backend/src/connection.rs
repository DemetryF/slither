use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::{broadcast, mpsc};

use core::SlitherID;

use crate::state_updater::ConnectionMessage;
use crate::transfer::AsyncReceive;

pub struct Connection {
    pub id: SlitherID,
    pub read_socket: OwnedReadHalf,

    pub directions_tx: mpsc::Sender<(SlitherID, f32)>,
    pub connections_tx: mpsc::Sender<ConnectionMessage>,
    pub crash_rx: broadcast::Receiver<SlitherID>,
}

impl Connection {
    pub async fn start(mut self) {
        let mut buffer = Vec::new();

        loop {
            if let Ok(id) = self.crash_rx.try_recv() {
                if self.id == id {
                    return;
                }
            }

            match protocol::ClientUpdate::receive(&mut buffer, &mut self.read_socket).await {
                Ok(protocol::ClientUpdate::Direction(dir)) => {
                    self.update_direction(dir).await;
                }

                Ok(protocol::ClientUpdate::Disconnect) => {
                    self.disconnect().await;
                    break;
                }

                Err(e) if e.kind() == tokio::io::ErrorKind::ConnectionReset => {
                    self.disconnect().await;
                    break;
                }

                _ => {}
            }
        }
    }

    async fn update_direction(&mut self, dir: f32) {
        self.directions_tx.send((self.id, dir)).await.unwrap();
    }

    async fn disconnect(self) {
        self.connections_tx
            .send(ConnectionMessage::Disconnected(self.id))
            .await
            .unwrap();
    }
}
