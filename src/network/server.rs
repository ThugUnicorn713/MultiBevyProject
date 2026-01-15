use bevy::ecs::system::command::insert_resource;
use bevy::prelude::*;
use bevy_renet2::prelude::*;
use bevy_renet2::prelude::{ConnectionConfig, ChannelConfig};
use bevy_renet2::netcode::{NetcodeServerTransport, ServerAuthentication, ServerSetupConfig};
use bevy_renet2::netcode::NativeSocket;

use std::net::UdpSocket;
use std::time::Duration;

use crate::network::messages::{ClientMessage, ServerMessage};
//use crate::game::player_movement::server_move_player;
use crate::game::player::Player;
use crate::network::constants::HOST_ID;
use rand::random;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {

        let (server, transport_layer) = setup_server();
        app
            .insert_resource(server)
            .insert_resource(transport_layer)
            .insert_resource(Lobby::default())
            .add_systems(Update, (update_server,));
    }
}

#[derive(Resource, Default)]
struct Lobby {

    players: Vec<u64>,
    max_players: usize,
}


fn setup_server() -> (RenetServer, NetcodeServerTransport) {
    let socket_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(socket_addr).unwrap();
    socket.set_nonblocking(true).unwrap();

      let channel = ChannelConfig {
        channel_id: 0,
        max_memory_usage_bytes: 1024 * 64, // 64 KB buffer
        send_type: SendType::ReliableOrdered{resend_time: Duration::from_millis(16)},
    };

    let connection_config = ConnectionConfig::from_channels(vec![channel.clone()], vec![channel]);
    let server = RenetServer::new(connection_config);

    let server_config = ServerSetupConfig {
        current_time: Duration::from_secs(1),
        max_clients: 3,
        protocol_id: 0,
        socket_addresses: vec![vec![socket_addr]],
        authentication: ServerAuthentication::Unsecure
    }; 

    let transport_layer = NetcodeServerTransport::new(server_config, NativeSocket::new(socket).unwrap()).unwrap();

    (server, transport_layer)      
}




fn update_server(
    mut server: ResMut<RenetServer>,
    mut transport: ResMut<NetcodeServerTransport>,
    mut lobby: ResMut<Lobby>,
    time:Res<Time>,
    mut commands: Commands,
    mut query: Query<(&Player, &mut Transform)>,
) {
    let _ = transport.update(time.delta(), &mut server);
    server.update(time.delta());

    // // --- Handle new host player manually ---
    // if !lobby.players.contains(&HOST_ID) {
    //     println!("Adding host player to lobby");
    //     lobby.players.push(HOST_ID);

    //     commands.spawn((
    //         Player { id: HOST_ID, speed: 5.0 },
    //         Transform::from_xyz(0.0, 0.5, 0.0),
    //         GlobalTransform::default(),
    //     ));
    // }

    let connected_clients: Vec<u64> = server.clients_id().into_iter().collect();

        for client_id in connected_clients.iter() {

            if lobby.players.contains(client_id){
                 continue;
            }

            println!("Client {} has Connected to Lobby", client_id);

            if lobby.players.len() >= 4 {
                println!("LOBBY FULL! Rejecting {}", client_id);

                server.disconnect(*client_id);
                continue;
            }

            lobby.players.push(*client_id);


          commands.spawn((
                Player { id: *client_id, speed: 5.0 },
                Transform::from_translation(Vec3::new(
                    (lobby.players.len() as f32) * 2.0, // spread out spawn positions
                    0.5,
                    0.0,
                )),

                GlobalTransform::default(),
            ));

            send_lobby_update(&mut server, &lobby);
        }

        let disconnected: Vec<u64> = lobby.players.iter().cloned()  //disconnetion handle logic 
            .filter(|id| !server.is_connected(*id)) // SECURITY!!!!! detects drops and silent disconnets
            .collect();

        for id in disconnected {
            println!("Client {} has been Yeeted!", id);

            lobby.players.retain(|p| *p != id);
            send_lobby_update(&mut server, &mut lobby);
        }

        for &client_id in &lobby.players{
            if client_id == HOST_ID {continue;}
                                                    //iter().cloned() {        //send client messages
            
            while let Some(message) = server.receive_message(client_id, 0) {

                let msg: ClientMessage = bincode::deserialize(&message).unwrap();
                match msg {
                    ClientMessage::JoinLobby => {
                        println!("Client {} has joined the lobby!", client_id);
                        send_lobby_update(&mut server, &lobby);
                    }

                    ClientMessage::PlayerInput { movement } => {
                        for (player, mut transform) in query.iter_mut() {
                      
                            if player.id == client_id {
    
                                    transform.translation += (movement * player.speed * time.delta_secs()).extend(0.0);
                            }
                        }
                    }
                }
                    
            }

                    // --- Send lobby update every frame so host + clients see all players ---
            let update = ServerMessage::LobbyUpdate(lobby.players.clone());
            let bytes = bincode::serialize(&update).unwrap();
            for &id in &lobby.players {server.send_message(id, 0, bytes.clone());} 
    }
}


fn send_lobby_update(
    server: &mut RenetServer, 
    lobby:  &Lobby,
) {
    let msg = ServerMessage::LobbyUpdate(lobby.players.clone());
    let p_amount = bincode::serialize(&msg).unwrap();

    for &client_id in lobby.players.iter() {
        server.send_message(client_id, 0, p_amount.clone()); // 0 channel, reliable and ordered, might add 1 as unreliable later
    }

    println!("Lobby Updated! Current Players {:?}", lobby.players);
}