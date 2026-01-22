use bevy::prelude::*;
use bevy::math::bounding::Aabb3d;
use serde::{Serialize, Deserialize};
use rand::Rng;

use crate::game::player::Player;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ObstacleData{
    pub position: Vec3,
    pub rotation: Quat,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub is_cylinder: bool,
    pub color: [f32; 3],
}

#[derive(Component)]
 pub struct Obstacle {

    pub aabb: Aabb3d, 
}

#[derive(Resource)]
pub struct MapData {

    pub obstacles: Vec<ObstacleData>,
}


pub fn generate_map() -> MapData {
    let mut rng = rand::thread_rng();
    let mut obstacles: Vec<ObstacleData> = Vec::new();

    let obstacle_count = 50;
    let map_size = 45.0;

    for _ in 0..obstacle_count {
        let width = rng.gen_range(2.0..8.0);
        let height = rng.gen_range(3.0..15.0);
        let depth = rng.gen_range(2.0..8.0);

        obstacles.push(ObstacleData { 
            position:  Vec3::new(
                rng.gen_range(-map_size..map_size),
                height / 2.0,
                rng.gen_range(-map_size..map_size),
            ),  
            rotation: Quat::from_rotation_y(rng.gen_range(0.0..std::f32::consts::TAU)),
            width,
            height, 
            depth, 
            is_cylinder: rng.gen_bool(0.5), 
            color: [
                rng.gen_range(0.1..0.4),
                rng.gen_range(0.1..0.4),
                rng.gen_range(0.1..0.4),
            ],
        });
    }
    println!("Generated map with {} obstacles", obstacles.len());
    MapData { obstacles }
    
}

pub fn spawn_map_obstacles(
    mut commands: Commands,
    map_data: Res<MapData>,
){
     for obstacle_data in &map_data.obstacles {
        let extents = Vec3::new(
            obstacle_data.width / 2.0,
            obstacle_data.height / 2.0,
            obstacle_data.depth / 2.0,
        );
        
        let aabb = Aabb3d {
            min: (-extents).into(),
            max: extents.into(),
        };
        
        commands.spawn((
            Obstacle { aabb },
            Transform {
                translation: obstacle_data.position,
                rotation: obstacle_data.rotation,
                ..default()
            },
            GlobalTransform::default(),
        ));
    }
    
    println!("SERVER: Spawned {} obstacle collision boxes", map_data.obstacles.len());
}


pub fn check_collisons(
    mut player_query: Query<(&Player, &mut Transform)>,
    obstacle_query: Query<(&Obstacle, &Transform), Without<Player>>,
){

    for (player, mut player_transform) in player_query.iter_mut(){
        let player_extents = Vec3::splat(2.5);

        let player_aabb = Aabb3d {
            min: (player_transform.translation - player_extents).into(),
            max: (player_transform.translation + player_extents).into(),
        };

         for (obstacle, obstacle_transform) in obstacle_query.iter(){
        let obstacle_pos = obstacle_transform.translation;

        let obstacle_aabb = Aabb3d {
            min: (Vec3::from(obstacle.aabb.min) + obstacle_pos).into(),
            max: (Vec3::from(obstacle.aabb.max) + obstacle_pos).into(),
        };

         if aabbs_intersect(&player_aabb, &obstacle_aabb) {
                let push_direction = (player_transform.translation - obstacle_pos).normalize();
                player_transform.translation += push_direction * 0.1;
            }
        }
    }

} 

fn aabbs_intersect(a: &Aabb3d, b: &Aabb3d) -> bool {
    let a_min = Vec3::from(a.min);
    let a_max = Vec3::from(a.max);
    let b_min = Vec3::from(b.min);
    let b_max = Vec3::from(b.max);
    
    a_min.x <= b_max.x && a_max.x >= b_min.x &&
    a_min.y <= b_max.y && a_max.y >= b_min.y &&
    a_min.z <= b_max.z && a_max.z >= b_min.z
}