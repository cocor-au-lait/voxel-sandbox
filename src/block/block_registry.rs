use bevy::prelude::*;
use super::block_type::BlockType;

/// インベントリに表示・使用可能なブロック (Air 除く)
pub const PLACEABLE_BLOCKS: &[BlockType] = &[
    BlockType::Stone,
    BlockType::Dirt,
    BlockType::Grass,
    BlockType::Sand,
    BlockType::Wood,
    BlockType::Leaves,
    BlockType::Cobblestone,
    BlockType::Planks,
    BlockType::Glass,
];

#[derive(Resource)]
pub struct BlockRegistry;

impl BlockRegistry {
    pub fn new() -> Self {
        Self
    }

    pub fn is_collidable(&self, block: BlockType) -> bool {
        !matches!(block, BlockType::Air | BlockType::Leaves | BlockType::Glass)
    }
}

impl Default for BlockRegistry {
    fn default() -> Self {
        Self::new()
    }
}
