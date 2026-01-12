use bevy::prelude::*;
use bevy_renet2::renet2::{RenetServer, RenetConnectionConfig, ServerAuthentication,};
use std::net::UdpSocket;

use crate::messages::{ClientMessage, ServerMessage};

pub struct ServerState {

    pub players: Vec<u64>,
    pub next_id: u64,
}