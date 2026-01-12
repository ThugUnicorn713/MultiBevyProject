use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet2::RepliconRenetPlugins;
use std::env;

fn is_host() -> bool {
    env::args().any(|arg| arg == "--host")
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, RepliconPlugins, RepliconRenetPlugins ));

    if host {
        app.insert_resource(ServerConfig::default());
        println!(" Starting as Host....");
    }else {
        app.insert_resource(ClientConfig::default());
        println!("Starting as Client.....")
    }

        .add_systems(Update, lobby_system)
        .run();
}

#[derive(Resource, Default)]
struct Lobby {
    players: Vec<u64>, //for player ids
}

#[derive(Component, Replicated)]
struct Player {
    id: u64,
}



fn lobby_system(
    mut lobby: ResMut<Lobby>,
    server: Option<Res<RepliconServer>>,
) {

    if let Some(server) = server {

        for event in server.connection_events() {
            match event {
                ConnectionEvent::Connected(client_id) => {
                    println!("Client {} has joined the lobby!", client_id);
                    lobby.players.push(client_id);
                }
                ConnectionEvent::Disconnected(client_id) => {
                    println!("Client {} has left the lobby!", client_id);
                    lobby.players.retain(|&id| id != client_id);
                }
            }
        }

        println!("There are {:?} players in lobby", lobby.players);

    }
}

