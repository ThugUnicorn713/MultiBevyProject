
  mod network;
  mod game;

use bevy::prelude::*;
use std::env;
use network::client::ClientPlugin;
use network::server::ServerPlugin;

fn is_host() -> bool {
    env::args().any(|arg| arg == "--host")
}

#[derive(Resource)]
pub struct HostFlag(pub bool);

fn main() {
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins);
    
    let host_flag = is_host();
    app.insert_resource(HostFlag(host_flag));

    if host_flag {
        app.add_plugins(ServerPlugin);
        app.add_plugins(ClientPlugin);
        println!("Starting as Host...");
    } else {
        app.add_plugins(ClientPlugin);
        println!("Starting as Client...");
    }
    
    app.run();
}