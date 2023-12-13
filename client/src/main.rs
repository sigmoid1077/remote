use bevy::{prelude::*, input::keyboard::KeyboardInput};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use net::{client::{Client, ClientPlugin}, ClientPacket};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(ClientPlugin::<ClientPacket>::default())
        .add_systems(Startup, startup)
        .add_systems(Update, update)
        .run()
}

fn startup(
    mut client: Client<ClientPacket>
) {
    client.connect("127.0.0.1:2560".parse().expect("Failed to parse specified address."));
}

fn update(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut client: Client<ClientPacket>
) {
    for keyboard_input_event in keyboard_input_events.read() {
        let packet = ClientPacket::Key(keyboard_input_event.key_code.unwrap() as u8, keyboard_input_event.state as u8);
        println!("Paclet: {:?}", packet);
        client.send_packet(packet);
    }
}
