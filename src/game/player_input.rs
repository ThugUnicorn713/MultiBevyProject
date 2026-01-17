use bevy::prelude::*;
use bevy::prelude::KeyCode;
use bevy_renet2::prelude::RenetClient;
use crate::HostFlag;
use crate::network::constants::HOST_ID;
use crate::network::messages::ClientMessage;
use crate::game::player::Player;


pub fn player_input_system(
    mut client: ResMut<RenetClient>,
    key: Res<ButtonInput<KeyCode>>,
    host_flag: Res<HostFlag>,
    mut query: Query<(&Player, &mut Transform)>,
    time: Res<Time>,

){
    let mut direction = Vec2::ZERO;

    if key.pressed(KeyCode::KeyW){
        direction.y += 1.0;
    }

    if key.pressed(KeyCode::KeyS){
        direction.y -= 1.0;
    }

    if key.pressed(KeyCode::KeyA){
        direction.x -= 1.0;
    }

    if key.pressed(KeyCode::KeyD){
        direction.x += 1.0;
    }

     if direction == Vec2::ZERO { return; }

     let movement_vec2 = direction.normalize();       // For sending to server



    let msg = bincode::serialize(&ClientMessage::PlayerInput { movement: movement_vec2 }).unwrap();
    client.send_message(0, msg);
    println!("Sending movement to server: {:?}", movement_vec2);

}   
   


