use bevy::prelude::*;
use bevy::prelude::KeyCode;
use bevy_renet2::prelude::RenetClient;
use crate::network::messages::ClientMessage;

pub fn player_input_system(
    mut client: ResMut<RenetClient>,
    key: Res<ButtonInput<KeyCode>>,
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

    if direction != Vec2::ZERO {

        let msg = ClientMessage::PlayerInput{

            movement: direction.normalize(),
        };

        let data = bincode::serialize(&msg).unwrap();
        client.send_message(0, data);
    }
}

