use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet2::RepliconRenetPlugins;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, RepliconPlugins, RepliconRenetPlugins ))
        .run();
}
