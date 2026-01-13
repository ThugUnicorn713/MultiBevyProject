use bevy::prelude::*;
use bevy_renet2::prelude::*;
use bevy_renet2::prelude::{ConnectionConfig, ChannelConfig};
use bevy_renet2::netcode::{ClientAuthentication, };
use std::net::UdpSocket;
use::std::time::Duration;
use crate::network::messages::{ClientMessage, ServerMessage};



pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(setup_client())
            .add_systems(Update, (update_client, send_join, receive_messages,));
    }
}



fn setup_client() -> RenetClient {
    let socket = UdpSocket::bind("127.0.0.1:5001").unwrap();
    socket.set_nonblocking(true).unwrap();

       let channel = ChannelConfig {
        channel_id: 0,
        max_memory_usage_bytes: 1024 * 64, // 64 KB buffer
        send_type: SendType::ReliableOrdered{resend_time: Duration::from_millis(16)},
    };

    let connection_config = ConnectionConfig::from_channels(vec![channel.clone()], vec![channel.clone()]);


    let authentication = ClientAuthentication::Unsecure {
        server_addr: "127.0.0.1:5000".parse().unwrap(),
        client_id: 0,
        protocol_id: 0,
        socket_id: 0,
        user_data: None,

    };

    RenetClient::new(connection_config,true)
}

fn update_client(mut client: ResMut<RenetClient>) {
    client.update(Duration::from_millis(16));
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



