use bevy::prelude::*;
use bevy_renet2::prelude::RenetServer;
use crate::network::messages::ClientMessage;
use crate::game::player::Player;
use std::time;


pub fn server_move_player(
    mut server: ResMut<RenetServer>,
    mut query: Query<(&Player, &mut Transform)>,
    time: Res<Time>,
){
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, 0) {

            let msg: ClientMessage = bincode::deserialize(&message).unwrap();

            if let ClientMessage::PlayerInput { movement } = msg {
                for (player, mut transform) in query.iter_mut() {
                    
                    if player.id == client_id {
                        
                        let speed = player.speed;
                        transform.translation += (movement * speed * time.delta_secs()).extend(0.0);
                    }
                }
            }

        }

    }
}