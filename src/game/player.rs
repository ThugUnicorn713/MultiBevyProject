use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use crate::HostFlag;
use crate::network::constants::HOST_ID;
use bevy::math::bounding::Aabb3d;

#[derive(Component)]
pub struct Player {

    pub id: u64,
    pub speed: f32,
}

#[derive(Component)]
pub struct HostCollider {
    pub aabb: Aabb3d,
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

     if direction == Vec2::ZERO { return; }

     let movement = Vec3::new(direction.x, 0.0, direction.y);
     
        for (player, mut transform) in query.iter_mut() {
            if player.id == HOST_ID {
                transform.translation += movement * player.speed * time.delta_secs();
                //println!("Host moved to: {:?}", transform.translation);
            }
    }


}

// pub fn rotate_host(
//     mut motion_events: EventReader<MouseMotion>,
//     mut query: Query<(&Player, &mut Transform)>,
//     host_flag: Res<HostFlag>,
// ){

//     if !host_flag.0 {return;}

//     let mut total_delta = Vec2::ZERO;

//     for event in motion_events.read(){
//         total_delta += event.delta;
//     }

//     if total_delta == Vec2::ZERO{ return;}

//     let sensitivity = 0.02;
//     let yaw = -total_delta.x * sensitivity;

//     for (player, mut transform) in query.iter_mut(){
//         if player.id == HOST_ID{

//             transform.rotate_y(yaw);
//         }
//     }

// }