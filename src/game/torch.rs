use bevy:: prelude::*;
//use bevy_pbr::*;
use bevy::ecs::system::EntityCommands;

pub fn spawn_torch(
    parent: &mut EntityCommands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
){

    parent.with_children(|builder|{

        builder.spawn((
        Mesh3d(meshes.add(Cylinder::new(1.0, 4.0))),
        MeshMaterial3d(materials.add(StandardMaterial {base_color: Color::srgb(0.5, 0.5, 0.5),
        metallic: 0.8,
            ..default()
        })),
        Transform::from_xyz(0.0,0.8 , -6.0)
             .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)), // Rotate 90 degrees!
        GlobalTransform::default(),

    ))
    .with_children(|flashlight| {
            flashlight.spawn((
                SpotLight {
                    intensity: 1500000.0,
                    color: Color::srgb(1.0, 1.0, 0.9),
                    range: 50000.0,
                    radius: 5.0,
                    shadows_enabled: true,
                    ..default()
                },
                Transform::from_xyz(0.0, -2.5, 0.0)
                    .looking_at(Vec3::new(0.0, -10.0, 0.0), Vec3::Z),
                GlobalTransform::default(),
            ));
        });
    });       
    
         
}