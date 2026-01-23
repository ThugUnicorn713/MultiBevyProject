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

    let wall_thickness = 2.0;
    let wall_height = 20.0;
    let wall_offset = map_size + wall_thickness / 2.0;

     //North, South, East, West collider walls for map

     obstacles.push(ObstacleData {
        position: Vec3::new(0.0, wall_height / 2.0, wall_offset),
        rotation: Quat::IDENTITY,
        width: map_size * 2.0 + wall_thickness * 2.0,
        height: wall_height,
        depth: wall_thickness,
        is_cylinder: false,
        color: [0.0, 0.0, 0.0], // invisible 
    });

    obstacles.push(ObstacleData {
        position: Vec3::new(0.0, wall_height / 2.0, -wall_offset),
        rotation: Quat::IDENTITY,
        width: map_size * 2.0 + wall_thickness * 2.0,
        height: wall_height,
        depth: wall_thickness,
        is_cylinder: false,
        color: [0.0, 0.0, 0.0],
    });

    obstacles.push(ObstacleData {
        position: Vec3::new(wall_offset, wall_height / 2.0, 0.0),
        rotation: Quat::IDENTITY,
        width: wall_thickness,
        height: wall_height,
        depth: map_size * 2.0 + wall_thickness * 2.0,
        is_cylinder: false,
        color: [0.0, 0.0, 0.0],
    });

    obstacles.push(ObstacleData {
        position: Vec3::new(-wall_offset, wall_height / 2.0, 0.0),
        rotation: Quat::IDENTITY,
        width: wall_thickness,
        height: wall_height,
        depth: map_size * 2.0 + wall_thickness * 2.0,
        is_cylinder: false,
        color: [0.0, 0.0, 0.0],
    });

    println!("Generated map with {} obstacles (including walls)", obstacles.len());
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

                // calculate overlap on each axis
                let player_min = Vec3::from(player_aabb.min);
                let player_max = Vec3::from(player_aabb.max);
                let obstacle_min = Vec3::from(obstacle_aabb.min);
                let obstacle_max = Vec3::from(obstacle_aabb.max);
                
                //calculate intersecting depth on each axis
                let overlap_x = (player_max.x - obstacle_min.x).min(obstacle_max.x - player_min.x);
                let overlap_z = (player_max.z - obstacle_min.z).min(obstacle_max.z - player_min.z);
                
                // push on X axis
                if overlap_x < overlap_z { 
                    if player_transform.translation.x < obstacle_pos.x {
                        player_transform.translation.x -= overlap_x; // push left
                    } else {
                        player_transform.translation.x += overlap_x; // push right
                    }
                }else {  //push on Z axis
                    if player_transform.translation.z < obstacle_pos.z {
                        player_transform.translation.z -= overlap_z; //push back
                    } else {
                        player_transform.translation.z += overlap_z; // push forward
                    }
                }

                //  let horizontal_offset = Vec2::new(
                //     player_transform.translation.x - obstacle_pos.x,
                //     player_transform.translation.z - obstacle_pos.z,
                // );
                
                // if horizontal_offset.length() > 0.0 {
                //     let push_direction_2d = horizontal_offset.normalize();
                //     let push_direction_3d = Vec3::new(push_direction_2d.x, 0.0, push_direction_2d.y);
                //     player_transform.translation += push_direction_3d * 0.2; 
                // }
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