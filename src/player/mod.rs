use bevy::prelude::*;

pub mod camera;

pub use camera::PlayerCamera;

#[derive(Component)]
pub struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, camera::spawn_player)
            .add_systems(Update, camera::fly_camera);
    }
}
