use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::prelude::KeyCode;
use bevy_renet2::prelude::RenetClient;

use crate::network::messages::ClientMessage;



pub fn player_input_system(
    mut client: ResMut<RenetClient>,
    key: Res<ButtonInput<KeyCode>>,
    mut motion_events: EventReader<MouseMotion>
){
    let mut direction = Vec2::ZERO;
     let mut mouse_delta = Vec2::ZERO;


    if key.pressed(KeyCode::KeyW){
        direction.y -= 1.0;
    }

    if key.pressed(KeyCode::KeyS){
        direction.y += 1.0;
    }

    if key.pressed(KeyCode::KeyA){
        direction.x -= 1.0;
    }

    if key.pressed(KeyCode::KeyD){
        direction.x += 1.0;
    }

     for event in motion_events.read() {
        mouse_delta += event.delta;
    }

     if direction == Vec2::ZERO && mouse_delta == Vec2::ZERO { return; }
     
     let rotation = -mouse_delta.x * 0.008;
     let movement = Vec3::new(direction.x, 0.0, direction.y);



    let msg = bincode::serialize(&ClientMessage::PlayerInput { movement: movement, rotation }).unwrap();
    client.send_message(0, msg);
    //println!("Sending movement to server: {:?}", movement);

}   
   


