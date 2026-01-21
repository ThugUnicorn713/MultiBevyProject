use bevy::prelude::*;
use bevy_renet2::prelude::*;
use bevy_renet2::prelude::{ConnectionConfig, ChannelConfig};
use bevy_renet2::netcode::{ClientAuthentication, NetcodeClientTransport, NativeSocket};
use std::net::UdpSocket;
use::std::time::Duration;
use std::time::SystemTime;
use crate::HostFlag;
use crate::network::messages::{ClientMessage, ServerMessage};
use crate::game::player_input::player_input_system;
use crate::game::torch::spawn_torch;
use crate::network::constants::HOST_ID;
use std::collections::HashSet;


#[derive(Component)]
pub struct RemotePlayer{

    pub id: u64,
    
}


pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        
        app
            
            .add_systems(Startup, (setup_host_client, set_scene))
            .add_systems(Update,
                 (
                    update_client, 
                    player_input_system, 
                    send_join, 
                    receive_messages,
                    flush_client_packets,
                )
                    .chain()
                    .run_if(has_client),  //seperates clients from host
                    );
    }
}

// fn debug_messages(mut client: ResMut<RenetClient>) {
//     while let Some(msg) = client.receive_message(0) {
//         println!("Client received raw bytes: {:?}", msg);
//     }
// }

fn has_client(client: Option<Res<RenetClient>>) -> bool { 
    client.is_some()
}

fn set_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut existing_ids: Local<HashSet<u64>>,
){

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 90.0, 90.0).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
    ));

    commands.spawn((PointLight {
        intensity: 5000.0,
        range: 1000.0,
        shadows_enabled: true,
        color:Color::WHITE,
      ..default()
     },
      Transform::from_xyz(4.0, 8.0, 4.0),
      GlobalTransform::default(),

    ));

    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(100.0, 100.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(0, 190, 0))),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        GlobalTransform::default(),
    ));

    existing_ids.insert(HOST_ID);
}

fn setup_host_client(mut commands: Commands, host_flag: Res<HostFlag>){
    if host_flag.0{
        return;
    }

    let (client, transport_layer) = setup_client();
            commands.insert_resource(client);
            commands.insert_resource(transport_layer);
}

fn setup_client() -> (RenetClient, NetcodeClientTransport) {
    let socket_addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let socket = UdpSocket::bind(socket_addr).unwrap();
    socket.set_nonblocking(true).unwrap();

       let channel = ChannelConfig {
        channel_id: 0,
        max_memory_usage_bytes: 5* 1024* 1024, //5mb
        send_type: SendType::ReliableOrdered{resend_time: Duration::from_millis(100)}, //16
    };
      

    let connection_config = ConnectionConfig{    
        client_channels_config: vec![channel.clone()],
        server_channels_config: vec![channel],
        available_bytes_per_tick: 16384, //default()
    };   

    let client = RenetClient::new(connection_config,false); //free up the socket, is restrictive, refused to connect if true
    let client_id =  { rand::random::<u64>() };


    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
   
    let authentication = ClientAuthentication::Unsecure {
        server_addr: "127.0.0.1:5000".parse().unwrap(),
        client_id,
        protocol_id: 0,
        socket_id: 0,
        user_data: None,

    };

    //println!("My ID: {:?}", client_id);

     let transport_layer = NetcodeClientTransport::new(
        current_time,
        authentication, 
        NativeSocket::new(socket).unwrap()).unwrap();

    (client, transport_layer)

}

fn update_client(
    mut client: ResMut<RenetClient>,
    mut transport: ResMut<NetcodeClientTransport>,
    time: Res<Time>,
) {

     match transport.update(time.delta(), &mut client) {
        Ok(_) => {},
        Err(e) => println!("CLIENT TRANSPORT ERROR: {:?}", e),
    }
    client.update(time.delta());
}

fn flush_client_packets(
    mut client: ResMut<RenetClient>,
    mut transport: ResMut<NetcodeClientTransport>,
) {
    let _ = transport.send_packets(&mut client);
} 

fn send_join(mut client: ResMut<RenetClient>, mut sent: Local<bool>, host_flag: Res<HostFlag>,) {
    if *sent || host_flag.0 { return; }

    if !client.is_connected() {
        return;
    }

    let msg = bincode::serialize(&ClientMessage::JoinLobby).unwrap();
    client.send_message(0, msg);

    *sent = true;

    println!("JoinLobby message sent");
}

fn receive_messages(mut client: ResMut<RenetClient>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut players: Query<(&RemotePlayer, &mut Transform)>,
    mut existing_ids: Local<HashSet<u64>>,
) {  

//let mut message_count = 0;
 while let Some(message) = client.receive_message(0) {
    //message_count += 1;
       // println!("CLIENT: Received message #{}, {} bytes", message_count, message.len());
        //println!("CLIENT: Received {} bytes", message.len());
        
        if let Ok(msg) = bincode::deserialize::<ServerMessage>(&message) {
           // println!("CLIENT: Deserialized: {:?}", msg);
            
            match msg {
                ServerMessage::LobbyUpdate(ids) => {
                     println!("CLIENT: Processing LobbyUpdate with {} players: {:?}", ids.len(), ids);
                    for (i, id) in ids.iter().enumerate() {
                        if existing_ids.contains(id) {
                             println!("  -> Player {} already exists, skipping", id);
                            continue;
                        }

                         println!("  -> Spawning new player {}", id);
                        let mut entity = commands.spawn((
                            RemotePlayer { id: *id },
                            Mesh3d(meshes.add(Cuboid::new(5.0, 5.0, 5.0))),
                            MeshMaterial3d(materials.add(
                                if *id == HOST_ID { 
                                    Color::srgb_u8(0, 0, 0) 
                                } else { 
                                    Color::srgb_u8(255, 255, 255) 
                                }
                            )),
                            Transform::from_xyz(i as f32 * 6.0, 0.5, 0.0),
                            GlobalTransform::default(),
                        ));
                        
                        if *id != HOST_ID {
                            spawn_torch(&mut entity, &mut meshes, &mut materials);
                        }

                        existing_ids.insert(*id);
                    }
                }

                ServerMessage::PlayerTransform { id, position, rotation } => {
                   //  println!("CLIENT: PlayerTransform for id={}, pos={:?}", id, position);
                    for (player, mut transform) in players.iter_mut() {
                        if player.id == id {
                            transform.translation = position;
                            transform.rotation = rotation;
                        }
                    }
                }

                ServerMessage::HostDetected { by_player_id, distance } => {
                     println!("CLIENT: Host spotted by player {} at {}m!", by_player_id, distance);
                }
            }
        }
    }
}




