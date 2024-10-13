use serde::{de::DeserializeOwned, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

impl AsyncReceive for protocol::PlayerJoin {}
impl AsyncReceive for protocol::ClientUpdate {}

impl AsyncSend for protocol::SessionStart {}
impl AsyncSend for protocol::ServerUpdate {}

pub trait AsyncReceive: DeserializeOwned {
    #[allow(async_fn_in_trait)]
    async fn receive(
        buffer: &mut Vec<u8>,
        reader: &mut (impl AsyncReadExt + std::marker::Unpin),
    ) -> tokio::io::Result<Self> {
        let packet_size = reader.read_u32().await?;

        buffer.clear();

        for _ in 0..packet_size {
            buffer.push(reader.read_u8().await?);
        }

        Ok(bincode::deserialize_from(buffer.as_slice()).unwrap())
    }
}

pub trait AsyncSend: Serialize {
    #[allow(async_fn_in_trait)]
    async fn send(
        &self,
        mut buffer: &mut Vec<u8>,
        writer: &mut (impl AsyncWriteExt + std::marker::Unpin),
    ) -> tokio::io::Result<()> {
        buffer.clear();

        bincode::serialize_into(&mut buffer, self).unwrap();

        let packet_size = buffer.len() as u32;

        writer.write_u32(packet_size).await?;
        writer.write_all(buffer.as_slice()).await?;

        Ok(())
    }
}
