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

     if direction == Vec2::ZERO { return; } // nothing pressed

     let movement_vec2 = direction.normalize();       // For sending to server
     let movement_vec3 = movement_vec2.extend(0.0);  // For moving transform locally


     if host_flag.0 {
        for (player, mut transform) in query.iter_mut() {
            if player.id == HOST_ID {
                transform.translation += movement_vec3 * player.speed * time.delta_secs();
                println!("Host moved to: {:?}", transform.translation);
            }
    }
} else {
    let msg = bincode::serialize(&ClientMessage::PlayerInput { movement: movement_vec2 }).unwrap();
    client.send_message(0, msg);
    println!("Sending movement to server: {:?}", movement_vec2);

}   
   
    }


