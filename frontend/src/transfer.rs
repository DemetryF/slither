use std::io::{Read, Write};

use serde::{de::DeserializeOwned, Serialize};

impl SyncSend for protocol::PlayerJoin {}
impl SyncSend for protocol::ClientUpdate {}

impl SyncReceive for protocol::SessionStart {}

pub trait SyncSend: Serialize {
    fn send(&self, mut buffer: &mut Vec<u8>, write: &mut impl Write) {
        buffer.clear();

        bincode::serialize_into(&mut buffer, self).unwrap();

        let packet_size = buffer.len() as u32;

        write.write_all(&packet_size.to_be_bytes()).unwrap();
        write.write_all(buffer.as_slice()).unwrap();
    }
}

pub trait SyncReceive: DeserializeOwned {
    fn receive(mut buffer: &mut Vec<u8>, reader: &mut impl Read) -> Self {
        buffer.clear();

        let packet_size = {
            let mut buffer = [0u8; 4];
            reader.read_exact(&mut buffer).unwrap();
            u32::from_be_bytes(buffer)
        };

        buffer.resize(packet_size as usize, 0);

        reader.read_exact(&mut buffer).unwrap();

        bincode::deserialize_from(buffer.as_slice()).unwrap()
    }
}
