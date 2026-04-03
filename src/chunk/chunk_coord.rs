use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_SIZE_I: i32 = CHUNK_SIZE as i32;
pub const CHUNK_SIZE_F: f32 = CHUNK_SIZE as f32;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkCoord(pub IVec3);

impl ChunkCoord {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }

    /// ワールド座標からチャンク座標を計算
    pub fn from_world_pos(pos: Vec3) -> Self {
        Self(IVec3::new(
            pos.x.div_euclid(CHUNK_SIZE_F) as i32,
            pos.y.div_euclid(CHUNK_SIZE_F) as i32,
            pos.z.div_euclid(CHUNK_SIZE_F) as i32,
        ))
    }

    /// チャンク原点のワールド座標
    pub fn world_origin(self) -> Vec3 {
        Vec3::new(
            self.0.x as f32 * CHUNK_SIZE_F,
            self.0.y as f32 * CHUNK_SIZE_F,
            self.0.z as f32 * CHUNK_SIZE_F,
        )
    }

    /// ワールド座標内のローカル座標 (0..CHUNK_SIZE)
    pub fn world_to_local(world_block: IVec3) -> UVec3 {
        UVec3::new(
            world_block.x.rem_euclid(CHUNK_SIZE_I) as u32,
            world_block.y.rem_euclid(CHUNK_SIZE_I) as u32,
            world_block.z.rem_euclid(CHUNK_SIZE_I) as u32,
        )
    }

    /// ローカル座標をワールドブロック座標に変換
    pub fn local_to_world(self, local: UVec3) -> IVec3 {
        IVec3::new(
            self.0.x * CHUNK_SIZE_I + local.x as i32,
            self.0.y * CHUNK_SIZE_I + local.y as i32,
            self.0.z * CHUNK_SIZE_I + local.z as i32,
        )
    }

    pub fn neighbors(self) -> [ChunkCoord; 6] {
        [
            ChunkCoord(self.0 + IVec3::X),
            ChunkCoord(self.0 - IVec3::X),
            ChunkCoord(self.0 + IVec3::Y),
            ChunkCoord(self.0 - IVec3::Y),
            ChunkCoord(self.0 + IVec3::Z),
            ChunkCoord(self.0 - IVec3::Z),
        ]
    }
}
