use bevy::prelude::*;

pub mod terrain_gen;

pub use terrain_gen::generate_flat_chunk;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, _app: &mut App) {}
}
