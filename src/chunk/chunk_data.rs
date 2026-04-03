use serde::{Deserialize, Serialize};
use crate::block::BlockType;
use super::chunk_coord::{CHUNK_SIZE, CHUNK_VOLUME};

/// チャンク内のボクセルデータ (32×32×32 = 32,768 バイト)
#[derive(Clone, Serialize, Deserialize)]
pub struct ChunkData {
    blocks: Vec<u8>,
}

impl ChunkData {
    pub fn new_empty() -> Self {
        Self {
            blocks: vec![BlockType::Air as u8; CHUNK_VOLUME],
        }
    }

    #[inline]
    fn index(x: usize, y: usize, z: usize) -> usize {
        y * CHUNK_SIZE * CHUNK_SIZE + z * CHUNK_SIZE + x
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockType {
        BlockType::try_from(self.blocks[Self::index(x, y, z)]).unwrap_or(BlockType::Air)
    }

    #[inline]
    pub fn get_ivec(&self, pos: bevy::prelude::IVec3) -> BlockType {
        if pos.x < 0
            || pos.y < 0
            || pos.z < 0
            || pos.x >= CHUNK_SIZE as i32
            || pos.y >= CHUNK_SIZE as i32
            || pos.z >= CHUNK_SIZE as i32
        {
            return BlockType::Air;
        }
        self.get(pos.x as usize, pos.y as usize, pos.z as usize)
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        self.blocks[Self::index(x, y, z)] = block as u8;
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.iter().all(|&b| b == BlockType::Air as u8)
    }

    pub fn raw(&self) -> &[u8] {
        &self.blocks
    }
}
