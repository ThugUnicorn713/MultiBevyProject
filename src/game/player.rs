use bevy::prelude::*;
use serde::{Serialize, Deserialize};

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
