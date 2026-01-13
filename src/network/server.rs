use bevy::prelude::*;
use bevy_renet2::renet2::{RenetServer, RenetConnectionConfig, ServerAuthentication,};
use std::net::UdpSocket;

use crate::messages::{ClientMessage, ServerMessage};

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(setup_server())
            .insert_resource(Lobby::default())
            .add_systems(Update, update_server);
    }
}

#[derive(Resource, Default)]
struct Lobby {

    players: Vec<u64>,
    max_players: usize,
}

fn setup_server() -> RenetServer {
    let socket = UdpSocket::bind("127.0.0.1:5000").unwrap();
    socket.set_unblocking(true).unwrap();

    RenetServer::new(
        RenetConnectionConfig::default(),
        socket,
        ServerAuthentication::Unsecure,
    )
}

fn update_server(
    mut server: ResMut<RenetServer>,
    mut lobby: ResMut<Lobby>,
) {
        while let Some(client_id) = server.accept_connection() {
            println!("Client {} has Connected to Lobby", client_id);

            if lobby.players.len() >= 4 {
                println!("LOBBY FULL! Rejecting {}", client_id);

                server.disconnect(client_id);
                continue;
            }

            lobby.players.push(client_id);
            send_lobby_update(&mut server, &lobby);
        }

        let disconnected: Vec<u64> = lobby.players.iter().cloned()  //disconnetion handle logic 
            .filter(|id| !server.is_client_connected(*id)) // SECURITY!!!!! detects drops and silent disconnets
            .collect();

        for id in disconnected {
            println!("Client {} has been Yeeted!", id);

            lobby.players.retain(|p| *p != id);
            send_lobby_update(&mut server, &mut lobby);
        }

        for client_id in lobby.players.iter().cloned() {        //send client messages
            while let Some(message) = server.receive_message(client_id, 0) {

                let msg: ClientMessage = bincode::deserialize(message).unwrap();
                match msg {
                    ClientMessage::JoinLobby => {
                        println!("Client {} has joined the lobby!", client_id);
                        send_lobby_update(&mut server, &mut lobby);
                    }
                }
            } 
        }

}

fn send_lobby_update(
    server: &mut RenetServer, 
    lobby:  &Lobby,
) {
    let msg = ServerMessage::LobbyUpdate(lobby.players.clone());
    let p_amount = bincode::serialize(&msg).unwrap();

    for &client_id in lobby.players.iter() {
        server.send_message(client_id, 0, p_amount.clone()); // 0 channel, reliable and ordered, might add 1 as unreleible later
    }

    println!("Lobby Updated! Current Players {:?}", lobby.players);
}
