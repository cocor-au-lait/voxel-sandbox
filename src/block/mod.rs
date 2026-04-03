use bevy::prelude::*;

pub mod block_registry;
pub mod block_type;

pub use block_registry::BlockRegistry;
pub use block_type::BlockType;

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BlockRegistry::new());
    }
}
