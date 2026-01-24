use serde::{Serialize, Deserialize};
use bevy::prelude::*;

use crate::game::map_gen::ObstacleData;
use crate::network::server::GameOutcome;


#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {

    JoinLobby,
    PlayerInput{ 
        movement: Vec3,
        rotation: f32,
    },

}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {

    LobbyUpdate(Vec<u64>),
    MapData(Vec<ObstacleData>),
    GameOver { outcome: GameOutcome },
    GameStarted,

    PlayerTransform{
        id: u64,
        position: Vec3,
        rotation: Quat,
    },

    HostDetected{
        by_player_id: u64, 
        distance: f32,
    },
 
    HostVisibility { 
        visible: bool,
        position: Option<Vec3>,
        rotation: Option<Quat>,

    },

    HostTransform { 
        position: Vec3, 
        rotation: Quat 
    },

    Gametimer {
        remaining: f32,
        total_visibility: f32,

    },



}