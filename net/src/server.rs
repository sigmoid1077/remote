use bevy::{prelude::*, ecs::system::SystemParam};
use crate::{Packet, TcpListenerComponent, TcpStreamComponent, BUF_SIZE};
use std::{io::Read, marker::PhantomData, net::{Ipv4Addr, Shutdown, SocketAddr, TcpListener}};

pub struct ServerPlugin<RecvPacket: Packet>(PhantomData<RecvPacket>);

impl<RecvPacket: Packet> Default for ServerPlugin<RecvPacket> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<RecvPacket: Packet> Plugin for ServerPlugin<RecvPacket> {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BindEvent>()
            .add_event::<UnbindEvent>()
            .add_event::<DisconnectedEvent>()
            .add_event::<RecvPacketEvent<RecvPacket>>()
            .add_systems(Update, (
                read_bind_event,
                read_unbind_event,
                accept_client_connections,
                write_disconnected_and_recv_packet_events::<RecvPacket>
            ));
    }
}

#[derive(SystemParam)]
pub struct Server<'w, 's, RecvPacket: Packet> {
    bind_event: EventWriter<'w, BindEvent>,
    unbind_event: EventWriter<'w, UnbindEvent>,
    recieved_packet_events: EventReader<'w, 's, RecvPacketEvent<RecvPacket>>
}

impl<'w, 's, RecvPacket: Packet> Server<'w, 's, RecvPacket> {
    pub fn bind(&mut self, port: u16) {
        self.bind_event.send(BindEvent(port));
    }

    pub fn unbind(&mut self) {
        self.unbind_event.send(UnbindEvent);
    }

    pub fn received_packets(&mut self) -> Vec<RecvPacket> {
        self.recieved_packet_events.read().map(|recv_packet_event| recv_packet_event.0.clone()).collect()
    }
}

#[derive(Event)]
pub(crate) struct BindEvent(pub(crate) u16);

#[derive(Event)]
pub(crate) struct UnbindEvent;

#[derive(Event)]
pub(crate) struct DisconnectedEvent;

#[derive(Event)]
pub(crate) struct RecvPacketEvent<RecvPacket: Packet>(pub(crate) RecvPacket);

pub(crate) fn read_bind_event(
    mut commands: Commands,
    mut bind_events: EventReader<BindEvent>,
    established_connections: Query<&TcpListenerComponent>
) {
    for bind_event in bind_events.read() {
        if established_connections.is_empty() {
            let tcp_listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, bind_event.0))).expect("Failed to bind to specified port.");
            tcp_listener.set_nonblocking(true).unwrap();
            commands.spawn(TcpListenerComponent(tcp_listener));
        }
    }
}

pub(crate) fn read_unbind_event(
    mut commands: Commands,
    connection: Query<&TcpStreamComponent>,
    client: Query<Entity, With<TcpStreamComponent>>,
    server: Query<Entity, With<TcpListenerComponent>>,
    mut unbind_events: EventReader<UnbindEvent>
) {
    for _unbind_event in unbind_events.read() {
        if let Ok(client) = client.get_single() {
            let connection = connection.single();
            let server = server.single();

            connection.0.shutdown(Shutdown::Both).expect("Failed to shutdown connection.");
            commands.entity(client).despawn();

            commands.entity(server).despawn();
        }
    }
}

pub(crate) fn accept_client_connections(
    mut commands: Commands,
    listener: Query<&TcpListenerComponent>
) {
    if let Ok(listener) = listener.get_single() {
        if let Ok((tcp_stream, _)) = listener.0.accept() {
            tcp_stream.set_nonblocking(true).unwrap();
            commands.spawn(TcpStreamComponent(tcp_stream));
        }
    }
}

pub(crate) fn write_disconnected_and_recv_packet_events<RecvPacket: Packet>(
    mut client_disconnected_event: EventWriter<DisconnectedEvent>,
    mut commands: Commands,
    client: Query<Entity, With<TcpStreamComponent>>,
    mut recieved_packet_event: EventWriter<RecvPacketEvent<RecvPacket>>,
    mut connection: Query<&mut TcpStreamComponent>
) {
    if let Ok(client) = client.get_single() {
        let mut connection = connection.single_mut();

        let mut buf = [0; BUF_SIZE];

        match connection.0.read(&mut buf) {
            Ok(0) => {
                client_disconnected_event.send(DisconnectedEvent);
                connection.0.shutdown(Shutdown::Both).expect("Failed to shutdown connection.");
                commands.entity(client).despawn();
            },
            Ok(len) => recieved_packet_event.send(RecvPacketEvent(RecvPacket::deserialize_packet(&buf[..len]))),
            _ => ()
        }
    }
}
