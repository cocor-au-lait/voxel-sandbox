use crate::block::BlockType;
use crate::chunk::{ChunkCoord, ChunkData, CHUNK_SIZE, CHUNK_SIZE_I};

pub const SEA_LEVEL: i32 = 4; // チャンク座標での海面 (ブロック座標 128)

/// Phase 1 用: フラットな地形を生成
/// y < 4 チャンク = 完全に石/土/草
/// y = 4 チャンク = 表面（高さ128）
pub fn generate_flat_chunk(coord: &ChunkCoord) -> ChunkData {
    let mut chunk = ChunkData::new_empty();
    let wy_base = coord.0.y * CHUNK_SIZE_I;

    for lx in 0..CHUNK_SIZE {
        for lz in 0..CHUNK_SIZE {
            for ly in 0..CHUNK_SIZE {
                let wy = wy_base + ly as i32;
                let block = if wy < 0 {
                    BlockType::Bedrock
                } else if wy < 60 {
                    BlockType::Stone
                } else if wy < 63 {
                    BlockType::Dirt
                } else if wy == 63 {
                    BlockType::Grass
                } else {
                    BlockType::Air
                };
                chunk.set(lx, ly, lz, block);
            }
        }
    }
    chunk
}
