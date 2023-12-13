use bevy::{prelude::*, ecs::system::SystemParam};
use crate::{Packet, TcpStreamComponent, BUF_SIZE};
use std::{io::{Read, Write}, marker::PhantomData, net::{Shutdown, SocketAddr, TcpStream}};

pub struct ClientPlugin<SendPacket: Packet>(pub PhantomData<SendPacket>);

impl<SendPacket: Packet> Default for ClientPlugin<SendPacket> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<SendPacket: Packet> Plugin for ClientPlugin<SendPacket> {
    fn build(&self, app: &mut App) {
        app
            .add_event::<ConnectEvent>()
            .add_event::<DisconnectEvent>()
            .add_event::<SendPacketEvent<SendPacket>>()
            .add_event::<UnboundEvent>()
            .add_systems(Update, (
                read_connect_event,
                read_disconnect_event,
                write_send_event::<SendPacket>,
                write_unbind_and_recv_packet_events
            ));
    }
}

#[derive(SystemParam)]
pub struct Client<'w, SendPacket: Packet> {
    connect_event: EventWriter<'w, ConnectEvent>,
    disconnect_event: EventWriter<'w, DisconnectEvent>,
    send_packet_event: EventWriter<'w, SendPacketEvent<SendPacket>>
}

impl<'w, 's, SendPacket: Packet> Client<'w, SendPacket> {
    pub fn connect(&mut self, socket_addr: SocketAddr) {
        self.connect_event.send(ConnectEvent(socket_addr));
    }

    pub fn disconnect(&mut self) {
        self.disconnect_event.send(DisconnectEvent);
    }
    
    pub fn send_packet(&mut self, packet: SendPacket) {
        self.send_packet_event.send(SendPacketEvent(packet));
    }
}

#[derive(Event)]
pub(crate) struct ConnectEvent(pub(crate) SocketAddr);

#[derive(Event)]
pub(crate) struct DisconnectEvent;

#[derive(Event)]
pub(crate) struct RecvPacketEvent<RecvPacket: Packet>(pub(crate) RecvPacket);

#[derive(Event)]
pub(crate) struct SendPacketEvent<SendPacket: Packet>(pub(crate) SendPacket);

#[derive(Event)]
pub(crate) struct UnboundEvent;

pub(crate) fn read_connect_event(
    mut commands: Commands,
    mut connect_events: EventReader<ConnectEvent>,
    established_connections: Query<&TcpStreamComponent>
) {
    for connect_event in connect_events.read() {
        if established_connections.is_empty() {
            let tcp_stream = TcpStream::connect(connect_event.0).expect("Failed to connect.");
            tcp_stream.set_nonblocking(true).unwrap();
            commands.spawn(TcpStreamComponent(tcp_stream));
        }
    }
}

pub(crate) fn read_disconnect_event(
    mut commands: Commands,
    connection: Query<&TcpStreamComponent>,
    mut disconnect_events: EventReader<DisconnectEvent>,
    client: Query<Entity, With<TcpStreamComponent>>
) {
    for _disconnect_event in disconnect_events.read() {
        let connection = connection.single();
        let client = client.single();
        
        connection.0.shutdown(Shutdown::Both).expect("Failed to shutdown connection.");
        commands.entity(client).despawn();
    }
}

pub(crate) fn write_send_event<SendPacket: Packet>(
    mut connection: Query<&mut TcpStreamComponent>,
    mut send_packet_events: EventReader<SendPacketEvent<SendPacket>>
) {
    for send_packet_event in send_packet_events.read() {
        let mut connection = connection.single_mut();
        
        connection.0.write_all(&send_packet_event.0.serialize_packet()).expect("Failed to send packet.");
    }
}

// TODO: Check if client receives server unbind packets
// even if the server will never send any packets.
// If it does, leave this function, otherwise remove it.
pub(crate) fn write_unbind_and_recv_packet_events(
    mut commands: Commands,
    mut connection: Query<&mut TcpStreamComponent>,
    client: Query<Entity, With<TcpStreamComponent>>,
    mut server_unbound_event: EventWriter<UnboundEvent>
) {
    if let Ok(mut connection) =  connection.get_single_mut() {
        let client = client.single();
    
        let mut buf = [0; BUF_SIZE];
    
        match connection.0.read(&mut buf) {
            Ok(0) => {
                server_unbound_event.send(UnboundEvent);
                connection.0.shutdown(Shutdown::Both).expect("Failed to shutdown connection.");
                commands.entity(client).despawn();
            }
            _ => ()
        }
    }
}
