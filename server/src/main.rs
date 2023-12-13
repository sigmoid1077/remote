use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use net::{ClientPacket, server::{Server, ServerPlugin}};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(ServerPlugin::<ClientPacket>::default())
        .add_systems(Startup, startup)
        .add_systems(Update, update)
        .run()
}

fn startup(
    mut server: Server<ClientPacket>
) {
    server.bind(2560);
}

fn update(
    mut server: Server<ClientPacket>
) {
    for packet in server.received_packets() {
        match packet {
            ClientPacket::Key(key_code, state) => println!("Key code: {}, State: {}", key_code, state),
            ClientPacket::Mouse => todo!()
        }
    }
}
