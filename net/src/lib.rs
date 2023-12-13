use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::net::{TcpStream, TcpListener};

pub mod client;
pub mod server;

pub(crate) const BUF_SIZE: usize = 1024;

#[derive(Component)]
pub(crate) struct TcpStreamComponent(pub(crate) TcpStream);

#[derive(Component)]
pub(crate) struct TcpListenerComponent(pub(crate) TcpListener);

pub trait Packet: Copy + for<'de> Deserialize<'de> + Send + Serialize + Sync + 'static {
    fn deserialize_packet(buffer: &[u8]) -> Self;
    fn serialize_packet(&self) -> Vec<u8>;
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ClientPacket {
    Key(u8, u8),
    Mouse
}

impl Packet for ClientPacket {
    fn serialize_packet(&self) -> Vec<u8> {
        bincode::serialize(&self).expect("Failed to serialize packet.")
    }

    fn deserialize_packet(buffer: &[u8]) -> Self {
        bincode::deserialize(buffer).expect("Failed to deserialize packet.")
    }
}
