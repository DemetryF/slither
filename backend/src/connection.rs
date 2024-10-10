use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc;

use core::SlitherID;

use crate::state_updater::ConnectionMessage;
use crate::transfer::AsyncReceive;

pub struct Connection {
    pub id: SlitherID,
    pub read_socket: OwnedReadHalf,

    pub directions_tx: mpsc::Sender<(SlitherID, f32)>,
    pub connections_tx: mpsc::Sender<ConnectionMessage>,
}

impl Connection {
    pub async fn start(mut self) {
        let mut buffer = Vec::new();

        loop {
            let packet = protocol::ClientUpdate::receive(&mut buffer, &mut self.read_socket).await;

            match packet {
                protocol::ClientUpdate::Direction(dir) => {
                    self.update_direction(dir).await;
                }

                protocol::ClientUpdate::Disconnect => {
                    self.disconnect().await;
                    break;
                }
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
