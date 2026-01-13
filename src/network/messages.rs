use serde::{Serialize, Deserialize};

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

}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {

    LobbyUpdate(Vec<u64>),

}