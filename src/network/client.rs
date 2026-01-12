use bevy::prelude::*;
use bevy_renet2::renet2::{ClientAuthentication, RenetClient, RenetConnectionConfig,};
use std::net::UdpSocket;
use crate::messages::{ClientMessage, ServerMessage};


pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(new_client())
            .add_systems(Update, (send_join, receive_messages));
    }
}



fn new_client() -> RenetClient {
    let socket = UdpSocket::bind("127.0.0.1.5001").unwrap();
    socket.set_unblocking(true).unwrap();

    let authentication = ClientAuthentication::Unsecure {
        server_addr: "127.0.0.1:5000".parse().unwrap(),
        client_id: None,
        protocol_id: 0,
    };

    RenetClient::new(
        RenetConnectionConfig::default(),
        socket,
        authentication,
    )
}

fn send_join(mut client: ResMut<RenetClient>, mut sent: Local<bool>) {
    if *sent { return; }

    let msg = bincode::serialize(&ClientMessage::JoinLobby).unwrap();
    client.send_message(0, msg);

    *sent = true;

    println!("JoinLobby message sent");
}

fn receive_messages(mut client: ResMut<RenetClient>) {
    while let Some(message) = client.receive_message(0) {
        let msg: ServerMessage = bincode::deserialize(&message).unwrap();
        println!("Server says: {:?}", msg);
    }
}



