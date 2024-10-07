use std::io::Write;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::io::AsyncReadExt;

#[derive(Serialize, Deserialize)]
pub enum ClientPacket {
    Direction(f32),
    Disconnect,
}

impl AsyncReceive for ClientPacket {}
impl SyncSend for ClientPacket {}

pub trait AsyncReceive: DeserializeOwned {
    #[allow(async_fn_in_trait)]
    async fn receive(
        buffer: &mut Vec<u8>,
        read: &mut (impl AsyncReadExt + std::marker::Unpin),
    ) -> Self {
        let packet_size = read.read_u32().await.unwrap();

        buffer.clear();

        for _ in 0..packet_size {
            buffer.push(read.read_u8().await.unwrap());
        }

        bincode::deserialize_from(buffer.as_slice()).unwrap()
    }
}

pub trait SyncSend: Serialize {
    fn send(&self, mut buffer: &mut Vec<u8>, write: &mut impl Write) {
        buffer.clear();

        bincode::serialize_into(&mut buffer, self).unwrap();

        let packet_size = buffer.len();

        write.write_all(&packet_size.to_be_bytes()).unwrap();
        write.write_all(buffer.as_slice()).unwrap();
    }
}
