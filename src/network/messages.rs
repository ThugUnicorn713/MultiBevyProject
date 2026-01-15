use serde::{Serialize, Deserialize};
use bevy::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum NetMessage {
    
    JoinRequest,
    PlayerConnected {id: u64},
    PlayerDisconnected {id: u64},
    LobbyState { players: Vec<u64> },

}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {

    JoinLobby,
    PlayerInput{ movement: Vec2,},
   // GhostAbility{is_cooldown_done: bool,ability_type: f32,},
    //BusterAbility{is_cooldown_done: bool, ability_type: f32,},

}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {

    LobbyUpdate(Vec<u64>),

}