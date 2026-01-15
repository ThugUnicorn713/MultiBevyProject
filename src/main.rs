
  mod network;
  mod game;

use bevy::prelude::*;
use std::env;
use network::client::ClientPlugin;
use network::server::ServerPlugin;

fn is_host() -> bool {
    env::args().any(|arg| arg == "--host")
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    if is_host() {
        app.add_plugins(ServerPlugin);
        println!("Starting as Host...");
    } else {
        app.add_plugins(ClientPlugin);
        println!("Starting as Client...");
    }
    
    app.run();
}