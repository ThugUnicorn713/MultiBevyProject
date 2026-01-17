use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use crate::HostFlag;
use crate::network::constants::HOST_ID;

#[derive(Component)]
pub struct Player {

    pub id: u64,
    pub speed: f32,
}

#[derive(Component, Serialize, Deserialize, Debug)]
pub enum Roles {

    Ghost,
    Buster,
}

pub fn move_host(
    key: Res<ButtonInput<KeyCode>>,
    host_flag: Res<HostFlag>,
    time: Res<Time>,
    mut query: Query<(&Player, &mut Transform)>,
){

    if !host_flag.0 { return;}

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

     let movement = direction.normalize();  // For moving transform locally
     
        for (player, mut transform) in query.iter_mut() {
            if player.id == HOST_ID {
                transform.translation += movement.extend(0.0) * player.speed * time.delta_secs();
                println!("Host moved to: {:?}", transform.translation);
            }
    }


}