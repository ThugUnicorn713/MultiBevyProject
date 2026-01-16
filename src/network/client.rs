use bevy::ecs::system::command;
use bevy::prelude::*;
use bevy_renet2::prelude::*;
use bevy_renet2::prelude::{ConnectionConfig, ChannelConfig};
use bevy_renet2::netcode::{ClientAuthentication, NetcodeClientTransport, NativeSocket};
use std::net::UdpSocket;
use::std::time::Duration;
use crate::HostFlag;
use crate::network::messages::{ClientMessage, ServerMessage};
use crate::game::player_input::player_input_system;
use crate::network::constants::HOST_ID;


#[derive(Component)]
pub struct RemotePlayer{

    pub id: u64,
}


pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        
        app
            
            .add_systems(Startup, (setup_host_client, set_scene))
            .add_systems(Update, (update_client, player_input_system, send_join,
                        spawn_players)
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
    mut materials: ResMut<Assets<StandardMaterial>>,){

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
    ));

    commands.spawn((PointLight {
        intensity: 5000.0,
        range: 500.0,
        shadows_enabled: true,
        ..default()
     },
      Transform::from_xyz(4.0, 8.0, 4.0),
      GlobalTransform::default(),

    ));

    commands.spawn((
    RemotePlayer { id: HOST_ID },
    Mesh3d(meshes.add(Cuboid::new(5.0,5.0,5.0))),
    MeshMaterial3d(materials.add(StandardMaterial { base_color: Color::WHITE, ..default() })),
    Transform::from_xyz(0.0,1.0,0.0),
    GlobalTransform::default(),
    ));

    //   //  DEBUG CUBE
    // commands.spawn((
    //     Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
    //     MeshMaterial3d(materials.add(StandardMaterial {
    //         base_color: Color::WHITE,
    //         ..default()
    //     })),
    //     Transform::from_xyz(0.0, 1.0, 0.0),
    //     GlobalTransform::default(),
    //));
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
        max_memory_usage_bytes: 1024 * 64, 
        send_type: SendType::ReliableOrdered{resend_time: Duration::from_millis(16)},
    };

    let connection_config = ConnectionConfig::from_channels(vec![channel.clone()], vec![channel]);

    let client = RenetClient::new(connection_config,true);
    let client_id =  { rand::random::<u64>() };
   
    let authentication = ClientAuthentication::Unsecure {
        server_addr: "127.0.0.1:5000".parse().unwrap(),
        client_id,
        protocol_id: 0,
        socket_id: 0,
        user_data: None,

    };

     let transport_layer = NetcodeClientTransport::new(
        Duration::from_millis(16), 
        authentication, 
        NativeSocket::new(socket).unwrap()).unwrap();

    (client, transport_layer)

}

fn update_client(
    mut client: ResMut<RenetClient>,
    mut transport: ResMut<NetcodeClientTransport>,
    time: Res<Time>,
) {
    let _ = transport.update(time.delta(), &mut client);
    client.update(time.delta());
}

fn send_join(mut client: ResMut<RenetClient>, mut sent: Local<bool>, host_flag: Res<HostFlag>,) {
    if *sent || host_flag.0 { return; }

    let msg = bincode::serialize(&ClientMessage::JoinLobby).unwrap();
    client.send_message(0, msg);

    *sent = true;

    println!("JoinLobby message sent");
}

// fn receive_messages(mut client: ResMut<RenetClient>) {
//     while let Some(message) = client.receive_message(0) {

//         let msg: ServerMessage = bincode::deserialize(&message).unwrap();
//         println!("Server says: {:?}", msg);
//     }
// } 

fn spawn_players(
    mut client: ResMut<RenetClient>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_players: Query<&RemotePlayer>, 
){
    while let Some(message) = client.receive_message(0){
        
        if let ServerMessage::LobbyUpdate(players) = bincode::deserialize(&message).unwrap() {
            println!("Lobby update received: {:?}", players);

         for (i,&id) in players.iter().enumerate() {
                if existing_players.iter().any(|p| p.id == id) { continue; }

                     commands.spawn((
                        RemotePlayer{id },
                        Mesh3d(meshes.add(Cuboid::new(5.0, 5.0, 5.0))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                                        base_color: Color::WHITE,
                                                ..default()
                                            })),
                        Transform::from_xyz(i as f32 * 6.0, 1.0, 0.0),
                        GlobalTransform::default(),
                     ));
           
            }

        }
    }
    
}