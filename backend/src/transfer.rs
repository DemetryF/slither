use serde::{de::DeserializeOwned, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

impl AsyncReceive for protocol::ClientUpdate {}

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

pub trait AsyncSend: Serialize {
    #[allow(async_fn_in_trait)]
    async fn send(
        &self,
        mut buffer: &mut Vec<u8>,
        writer: &mut (impl AsyncWriteExt + std::marker::Unpin),
    ) {
        buffer.clear();

        bincode::serialize_into(&mut buffer, self).unwrap();

        let packet_size = buffer.len() as u32;

        writer.write_u32(packet_size).await.unwrap();
        writer.write_all(buffer.as_slice()).await.unwrap();
    }
}
