use bevy::math::bounding::Aabb3d;
use bevy::math::bounding::RayCast3d;
use bevy::prelude::*;
use bevy_renet2::prelude::*;
use bevy_renet2::prelude::{ConnectionConfig, ChannelConfig, ServerEvent};
use bevy_renet2::netcode::{NetcodeServerTransport, ServerAuthentication, ServerSetupConfig};
use bevy_renet2::netcode::NativeSocket;
 

use std::net::UdpSocket;
use std::time::Duration;
use std::time::SystemTime;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::HostFlag;
use crate::network::messages::{ClientMessage, ServerMessage};
use crate::game::player::*;
use crate::game::player::move_host;
use crate::game::torch::*;
use crate::game::map_gen::*;

use crate::network::constants::HOST_ID;


pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {

        let (server, transport_layer) = setup_server();
        let map_data = generate_map();

        app
            .insert_resource(server)
            .insert_resource(transport_layer)
            .insert_resource(Lobby::default())
            .insert_resource(HostSpottedTracker::default())
            .insert_resource(HostVisibilityState::default())
            .insert_resource(map_data)
            .add_event::<HostDetected>()
            .add_systems(Startup, (spawn_host_entity, spawn_map_obstacles, spawn_visual_obstacles_for_host))
            .add_systems(Update, (update_server, flashlight_detection, 
                track_host_spotting, update_host_visibility, host_detection_handler, check_collisons))
            .add_systems(Update, move_host.run_if(|host_flag: Res<HostFlag>| host_flag.0));
    }
}

#[derive(Event)]
pub struct HostDetected {
    pub detected_by_player_id: u64,
    pub distance: f32,
}

#[derive(Resource, Default)]
struct HostSpottedTracker {
   spotted_timers: HashMap<u64, f32>, //player id and time spotted
}

#[derive(Resource)]
struct HostVisibilityState {
    is_visible: bool,
    vis_timer: f32,
    vis_duration: f32,
}

impl Default for HostVisibilityState {
    fn default() -> Self {
        Self { 
            is_visible: false,
            vis_timer: 0.0, 
            vis_duration: 3.0 
        }    
    }
}


#[derive(Resource)]
struct Lobby {

    players: Vec<u64>,
    confirmed: Vec<u64>,
}

impl Default for Lobby {   
    fn default() -> Self {
        Self {
            players: vec![HOST_ID],
            confirmed: vec![HOST_ID],
        }
    }
}

fn setup_server() -> (RenetServer, NetcodeServerTransport) {
    let socket_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(socket_addr).unwrap();
    socket.set_nonblocking(true).unwrap();

      let channel = ChannelConfig {
        channel_id: 0,
        max_memory_usage_bytes: 5 * 1024 * 1024, 
        send_type: SendType::ReliableOrdered{resend_time: Duration::from_millis(100)},
      };

      

    let connection_config = ConnectionConfig {   
        server_channels_config: vec![channel.clone()],
        client_channels_config: vec![channel],
        available_bytes_per_tick: 16384,
    };                                                                
    let server = RenetServer::new(connection_config);


    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let server_config = ServerSetupConfig {
        current_time,
        max_clients: 3,
        protocol_id: 0,
        socket_addresses: vec![vec![socket_addr]],
        authentication: ServerAuthentication::Unsecure
    }; 

    let transport_layer = NetcodeServerTransport::new(server_config, NativeSocket::new(socket).unwrap()).unwrap();

    (server, transport_layer)     

    
}

fn spawn_host_entity(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){

        commands.spawn((
        Player { id: HOST_ID, speed: 8.0 },
        Mesh3d(meshes.add(Cuboid::new(5.0, 5.0, 5.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(0, 0, 0))), // Black for host
        Transform::from_xyz(0.0, 2.5, 0.0),
        GlobalTransform::default(),
        HostCollider{aabb:Aabb3d{min: Vec3::splat(-2.5).into(), max: Vec3::splat(2.5).into(),}},
    ));
} 

fn spawn_visual_obstacles_for_host(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map_data: Res<MapData>,
){
    for obstacle in &map_data.obstacles {
        let mesh = if obstacle.is_cylinder {
            meshes.add(Cylinder::new(obstacle.width / 2.0,obstacle.height))
        } else {
            meshes.add(Cuboid::new(obstacle.width, obstacle.height, obstacle.depth))
        };

        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(materials.add(StandardMaterial{ 
                base_color: Color::srgb(obstacle.color[0], obstacle.color[1], obstacle.color[2]),
                ..default()
            })),
            Transform{
                translation: obstacle.position,
                rotation: obstacle.rotation,
                ..default()
            },
            GlobalTransform::default(),
        ));
    }

}

// fn flush_server_packets(
//    mut server: ResMut<RenetServer>,
//    mut transport: ResMut<NetcodeServerTransport>, 
// ){
//      let _ = transport.send_packets(&mut server);
// }



fn update_server(
    mut server: ResMut<RenetServer>,
    mut transport: ResMut<NetcodeServerTransport>,
    mut lobby: ResMut<Lobby>,
    mut commands: Commands,
    mut query: Query<(&Player, &mut Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visibility: Res<HostVisibilityState>,
    map_data: Res<MapData>,
    time:Res<Time>,
) {
    
     let _ = transport.update(time.delta(), &mut server);
    server.update(time.delta());
  
    // for client_id in server.clients_id() {
    //     let stats = server.network_info(client_id).unwrap();
    //    // println!("SERVER stats for client {}: sent_bytes={}, received_bytes={}", client_id, stats.bytes_sent_per_second, stats.bytes_received_per_second);
    // }
    

    // Handle connection/disconnection events
    while let Some(event) = server.get_event() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("═══════════════════════════════════════");
                println!("SERVER EVENT: Client {} connected", client_id);
                println!("SERVER: Is client connected? {}", server.is_connected(client_id));
                
                if !lobby.players.contains(&client_id) {
                    lobby.players.push(client_id);
                    println!("SERVER: Added client {} to lobby.players", client_id);

                    // Spawn their player entity
                   let mut entity = commands.spawn((
                        Player { id: client_id, speed: 5.0 },
                        Mesh3d(meshes.add(Cuboid::new(5.0, 5.0, 5.0))),
                        MeshMaterial3d(materials.add(Color::srgb_u8(255, 255, 255))),
                        Transform::from_xyz((lobby.players.len() as f32) * 6.0, 2.5, 0.0),
                        GlobalTransform::default(),
                    ));
                        
                        spawn_torch(&mut entity, &mut meshes, &mut materials);

                     println!("SERVER: Spawned player entity for client {}", client_id);

                        //send map data!
                        let map_msg = ServerMessage::MapData(map_data.obstacles.clone());
                        let map_bytes = bincode::serialize(&map_msg).unwrap();

                        server.send_message(client_id, 0, map_bytes);
                        println!("SERVER: Sent map data to client {}", client_id);

                     send_lobby_update(&mut server, &lobby);
                }

                 println!("═══════════════════════════════════════");
            }
            
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("SERVER EVENT: Client {} disconnected: {:?}", client_id, reason);
                lobby.players.retain(|p| *p != client_id);
                lobby.confirmed.retain(|p| *p != client_id);
            }
        }
    }


    // Process messages from all connected clients

    for client_id in server.clients_id() {

        while let Some(message) = server.receive_message(client_id, 0) { 
           // println!("SERVER: Received {} bytes from client {}", message.len(), client_id);
            
            if let Ok(msg) = bincode::deserialize::<ClientMessage>(&message) {
               // println!("SERVER: Deserialized: {:?}", msg);

                match msg {
                    ClientMessage::JoinLobby => {
                        println!("SERVER: Client {} joined lobby", client_id);
                        
                        if !lobby.confirmed.contains(&client_id) {
                            lobby.confirmed.push(client_id);
                            send_lobby_update(&mut server, &lobby);
                            println!("SERVER: Added {} to confirmed list", client_id)
                        }
                    }
                    
                    ClientMessage::PlayerInput { movement, rotation } => {
                        for (player, mut transform) in query.iter_mut() {
                            if player.id == client_id {
                                transform.translation += movement * player.speed * time.delta_secs();
                                transform.rotate_y(rotation);
                                //player.rotation += rotation;
                            }
                        }
                    }
                }
            }
        }
    }

    // Broadcast player transforms to all clients
    for (player, transform) in query.iter() {
        let msg = ServerMessage::PlayerTransform {
            id: player.id,
            position: transform.translation,
            rotation: transform.rotation,
        };

        let data = bincode::serialize(&msg).unwrap();

        for client_id in server.clients_id() {
            server.send_message(client_id, 0, data.clone());
            
        }
    }

    if visibility.is_visible {
        for (player, transform) in query.iter(){

            if player.id == HOST_ID {
                let msg = ServerMessage::HostTransform {
                    position: transform.translation,
                    rotation: transform.rotation,
                };
                let data = bincode::serialize(&msg).unwrap();

                for client_id in server.clients_id() {
                    server.send_message(client_id, 0, data.clone());
                }
                break;
            }

        }

    }

        let _ = transport.send_packets(&mut server); //THE HERO!!!
        
    }



fn send_lobby_update(
    server: &mut RenetServer, 
    lobby:  &Lobby,
) {
    
    // let  all_players = lobby.players.clone();

    // println!("SERVER: Sending lobby update with {} players: {:?}", all_players.len(), all_players);

    // let msg = ServerMessage::LobbyUpdate(all_players);

    // let p_amount = bincode::serialize(&msg).unwrap();

    // Filter out HOST_ID - clients won't spawn host initially
    let visible_players: Vec<u64> = lobby.players.iter()
        .filter(|&&id| id != HOST_ID)
        .copied()
        .collect();

    println!("SERVER: Sending lobby update with {} players (excluding host): {:?}", visible_players.len(), visible_players);

    let msg = ServerMessage::LobbyUpdate(visible_players);
    let p_amount = bincode::serialize(&msg).unwrap();

    for client_id in server.clients_id() {
        server.send_message(client_id, 0, p_amount.clone());
         println!("  -> Sent to client {}", client_id);                                  // 0 channel, reliable and ordered, might add 1 as unreliable later
    }

}

fn flashlight_detection(
    flash_query: Query<(&GlobalTransform, &Flashlight, &ChildOf)>,
    player_query: Query<&Player>,
    host_query: Query<(&GlobalTransform, &Player, &HostCollider)>,
    obstacle_query: Query<(&Obstacle, &Transform)>,
    mut detect_events: EventWriter<HostDetected>
){
    let Some((host_transform, host_player, host_collider)) = host_query.iter()
        .find(|(_, player, _)| player.id == HOST_ID)
    else{
        return;
    };

    let host_pos: Vec3 = host_transform.translation();
    let world_aabb = Aabb3d{
        min: (Vec3::from(host_collider.aabb.min) + host_pos).into(),
        max: (Vec3::from(host_collider.aabb.max) + host_pos).into(),
    };
     
    //check each flashlight
    for (torch_transform, flashlight, child_of) in flash_query.iter() {
        let parent_entity = child_of.parent();

        let Ok(player) = player_query.get(parent_entity) else {
            continue;
        };

         if player.id == HOST_ID {
            continue;
        }

    
        let ray_origin = torch_transform.translation();
        let ray_direct = torch_transform.down();
        let raycast = RayCast3d::new(ray_origin, ray_direct, flashlight.range);

        if let Some(host_distance) = raycast.aabb_intersection_at(&world_aabb) {
        
            let mut is_blocked = false;
            
            for (obstacle, obstacle_transform) in obstacle_query.iter() {
                let obstacle_pos = obstacle_transform.translation;
                let obstacle_aabb = Aabb3d {
                    min: (Vec3::from(obstacle.aabb.min) + obstacle_pos).into(),
                    max: (Vec3::from(obstacle.aabb.max) + obstacle_pos).into(),
                };
                
                // Check if obstacle blocks the ray
                if let Some(obstacle_distance) = raycast.aabb_intersection_at(&obstacle_aabb) {
                    if obstacle_distance < host_distance {
                        is_blocked = true;
                        break;
                    }
                }
            }
            
            if !is_blocked {
                detect_events.write(HostDetected {
                    detected_by_player_id: player.id,
                    distance: host_distance,
                });
            }
        }
    }

}

fn host_detection_handler(
    mut events: EventReader<HostDetected>,
    mut server: ResMut<RenetServer>,
){
     for event in events.read() {
        println!("Host detected by player {} at distance {}", 
                 event.detected_by_player_id, event.distance);
                

     let msg = ServerMessage::HostDetected {
            by_player_id: event.detected_by_player_id,
            distance: event.distance,
        };

        let data = bincode::serialize(&msg).unwrap();

        for client_id in server.clients_id() {
            server.send_message(client_id, 0, data.clone());
            
        }
    }
}      

fn track_host_spotting(
    mut events: EventReader<HostDetected>,
    mut tracker: ResMut<HostSpottedTracker>,
    mut visibility: ResMut<HostVisibilityState>,
    time: Res<Time>,
) {
    let mut spotting_this_frame = HashSet::new();
    
    for event in events.read() {
        spotting_this_frame.insert(event.detected_by_player_id); //collect who is spotting

        let timer = tracker.spotted_timers
            .entry(event.detected_by_player_id)
            .or_insert(0.0);
        
        *timer += time.delta_secs();
        
        println!("DEBUG: Player {} spotting timer now at: {} seconds", event.detected_by_player_id, *timer);
        
        if *timer >= 3.0 && !visibility.is_visible {
            println!("Host has been spotted for 3 secs! Making visible!");
            visibility.is_visible = true;
            visibility.vis_timer = visibility.vis_duration;
        }
    }
    
    // Decay timers ONLY for players NOT spotting this frame
    for (player_id, timer) in tracker.spotted_timers.iter_mut() {
        if !spotting_this_frame.contains(player_id) {
            if *timer > 0.0 {
                println!("DEBUG: Player {} looked away! Timer is reset!", player_id);
            }
            *timer = 0.0;
        }
    }
    
    //println!("DEBUG: Processed {} detection events this frame. Visibility: {}", spotting_this_frame.len(), visibility.is_visible);
}


//manage ghost visibility timer 
fn update_host_visibility(    
    mut visibility: ResMut<HostVisibilityState>,
    mut server: ResMut<RenetServer>,
    host_query: Query<(&Transform, &Player)>,
    time: Res<Time>,
) {

    let has_became_visible = visibility.is_visible && 
                               visibility.vis_timer == visibility.vis_duration;
    
    if has_became_visible {
        println!("Host becoming visible to clients!");
       
        
        
        
        for (transform, player) in host_query.iter() {
            if player.id == HOST_ID {
                let vis_msg = ServerMessage::HostVisibility { 
                    visible: true,
                    position: Some(transform.translation),
                    rotation: Some(transform.rotation), 
                };
                let vis_data = bincode::serialize(&vis_msg).unwrap();
                // let transform_msg = ServerMessage::HostTransform {
                //     position: transform.translation,
                //     rotation: transform.rotation,
                // };
                //let transform_data = bincode::serialize(&transform_msg).unwrap();
                
                for client_id in server.clients_id() {
                    server.send_message(client_id, 0, vis_data.clone());
                    //server.send_message(client_id, 0, transform_data.clone());
                }
            }
        }

    }
    
    
    if visibility.is_visible {
        visibility.vis_timer -= time.delta_secs();

        if visibility.vis_timer <= 0.0 {
            println!("Host becoming invisible again!");
            visibility.is_visible = false;
            
            let msg = ServerMessage::HostVisibility { 
                visible: false,
                position: None,
                rotation: None, 
            };
            let data = bincode::serialize(&msg).unwrap();
            
            for client_id in server.clients_id() {
                server.send_message(client_id, 0, data.clone());
            }
        }
    }
}

