use bevy::prelude::*;

pub mod terrain_gen;

pub use terrain_gen::TerrainGenerator;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerrainGenerator::new(42));
    }
}
