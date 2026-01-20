use serde::{Serialize, Deserialize};
use bevy::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {

    JoinLobby,
    PlayerInput{ 
        movement: Vec3,
        rotation: f32,
    },
   // GhostAbility{is_cooldown_done: bool,ability_type: f32,},
    //BusterAbility{is_cooldown_done: bool, ability_type: f32,},

}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {

    LobbyUpdate(Vec<u64>),

    PlayerTransform{
        id: u64,
        position: Vec3,
        rotation: Quat,
    }
}