use bevy::prelude::*;
use std::collections::HashMap;

pub mod chunk_coord;
pub mod chunk_data;
pub mod chunk_manager;

pub use chunk_coord::{ChunkCoord, CHUNK_SIZE, CHUNK_SIZE_F, CHUNK_SIZE_I, CHUNK_VOLUME};
pub use chunk_data::ChunkData;
pub use chunk_manager::{GenerationQueue, LastPlayerChunk};

/// チャンク座標 → Entity の対応表
#[derive(Resource, Default)]
pub struct ChunkMap(pub HashMap<IVec3, Entity>);

/// チャンク座標 → ボクセルデータ
#[derive(Resource, Default)]
pub struct ChunkDataStore(pub HashMap<IVec3, ChunkData>);

/// プレイヤーが変更したチャンク (セーブ対象)
#[derive(Resource, Default)]
pub struct ModifiedChunks(pub std::collections::HashSet<IVec3>);

/// チャンクエンティティのマーカー
#[derive(Component)]
pub struct ChunkEntity;

/// メッシュ再生成フラグ
#[derive(Component)]
pub struct ChunkMeshDirty;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkMap>()
            .init_resource::<ChunkDataStore>()
            .init_resource::<ModifiedChunks>()
            .init_resource::<GenerationQueue>()
            .init_resource::<LastPlayerChunk>()
            .add_systems(
                Update,
                (
                    chunk_manager::update_loaded_chunks,
                    chunk_manager::generate_queued_chunks,
                    chunk_manager::sync_chunk_entities,
                )
                    .chain(),
            );
    }
}
