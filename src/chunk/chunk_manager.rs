use bevy::prelude::*;
use std::collections::VecDeque;

use crate::chunk::{ChunkCoord, ChunkDataStore, ChunkEntity, ChunkMap};
use crate::meshing::MeshingQueue;
use crate::player::Player;
use crate::terrain::TerrainGenerator;

/// 描画距離 (水平, チャンク単位)
pub const RENDER_DISTANCE: i32 = 6;
/// 垂直方向の描画範囲 (チャンク単位, 0 を中心に ± この値)
pub const VERTICAL_RANGE: i32 = 3;
/// 破棄距離 = 描画距離 + ヒステリシス
pub const UNLOAD_DISTANCE: i32 = RENDER_DISTANCE + 2;

/// チャンク生成待ちキュー
#[derive(Resource, Default)]
pub struct GenerationQueue(pub VecDeque<IVec3>);

/// プレイヤーの前フレームチャンク座標
#[derive(Resource, Default)]
pub struct LastPlayerChunk(pub Option<IVec3>);

/// 毎フレーム: プレイヤー周辺のチャンクを生成/破棄管理
pub fn update_loaded_chunks(
    player_q: Query<&Transform, With<Player>>,
    mut chunk_map: ResMut<ChunkMap>,
    mut chunk_store: ResMut<ChunkDataStore>,
    mut gen_queue: ResMut<GenerationQueue>,
    mut last_chunk: ResMut<LastPlayerChunk>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_chunk = ChunkCoord::from_world_pos(player_tf.translation).0;

    // プレイヤーがチャンクを移動したときのみ更新
    if last_chunk.0 == Some(player_chunk) {
        return;
    }
    last_chunk.0 = Some(player_chunk);

    // 破棄: 遠方チャンクを削除 (EntityはChunkSpawnシステムで管理)
    chunk_store.0.retain(|&coord, _| {
        let dx = (coord.x - player_chunk.x).abs();
        let dy = (coord.y - player_chunk.y).abs();
        let dz = (coord.z - player_chunk.z).abs();
        dx <= UNLOAD_DISTANCE && dy <= UNLOAD_DISTANCE + 2 && dz <= UNLOAD_DISTANCE
    });
    chunk_map.0.retain(|&coord, _| {
        let dx = (coord.x - player_chunk.x).abs();
        let dy = (coord.y - player_chunk.y).abs();
        let dz = (coord.z - player_chunk.z).abs();
        dx <= UNLOAD_DISTANCE && dy <= UNLOAD_DISTANCE + 2 && dz <= UNLOAD_DISTANCE
    });

    // 生成: 描画距離内で未ロードのチャンクをキューに追加
    let mut candidates = Vec::new();
    for dx in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for dz in -RENDER_DISTANCE..=RENDER_DISTANCE {
            for dy in -VERTICAL_RANGE..=VERTICAL_RANGE {
                let coord = player_chunk + IVec3::new(dx, dy, dz);
                if !chunk_map.0.contains_key(&coord) && !chunk_store.0.contains_key(&coord) {
                    let dist2 = dx * dx + dy * dy + dz * dz;
                    candidates.push((dist2, coord));
                }
            }
        }
    }
    candidates.sort_by_key(|(d, _)| *d);

    for (_, coord) in candidates {
        gen_queue.0.push_back(coord);
    }
}

/// 毎フレーム: キューから最大 N 個のチャンクを生成
pub fn generate_queued_chunks(
    mut gen_queue: ResMut<GenerationQueue>,
    mut chunk_store: ResMut<ChunkDataStore>,
    mut meshing_queue: ResMut<MeshingQueue>,
    terrain_gen: Res<TerrainGenerator>,
) {
    const GENERATE_BUDGET: usize = 4;
    let mut count = 0;

    while let Some(coord) = gen_queue.0.pop_front() {
        if count >= GENERATE_BUDGET {
            break;
        }
        if chunk_store.0.contains_key(&coord) {
            continue;
        }
        let chunk_coord = ChunkCoord(coord);
        let chunk_data = terrain_gen.generate_chunk(&chunk_coord);
        chunk_store.0.insert(coord, chunk_data);
        meshing_queue.0.push_back(coord);
        count += 1;
    }
}

/// 毎フレーム: チャンクEntityのスポーン/デスポーン
pub fn sync_chunk_entities(
    mut commands: Commands,
    mut chunk_map: ResMut<ChunkMap>,
    chunk_store: Res<ChunkDataStore>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_material: Res<crate::rendering::ChunkMaterial>,
    chunk_entity_q: Query<Entity, With<ChunkEntity>>,
) {
    // 破棄: chunk_store に存在しなくなったEntityを削除
    let existing_coords: std::collections::HashSet<IVec3> =
        chunk_map.0.keys().copied().collect();
    for coord in existing_coords {
        if !chunk_store.0.contains_key(&coord) {
            if let Some(entity) = chunk_map.0.remove(&coord) {
                commands.entity(entity).despawn();
            }
        }
    }

    // スポーン: chunk_store にあって Entity がまだないチャンク
    for (&coord, _) in chunk_store.0.iter() {
        if chunk_map.0.contains_key(&coord) {
            continue;
        }

        let chunk_pos = ChunkCoord(coord).world_origin();
        let entity = commands
            .spawn((
                ChunkEntity,
                ChunkCoord(coord),
                Mesh3d(meshes.add(Mesh::new(
                    bevy::mesh::PrimitiveTopology::TriangleList,
                    bevy::asset::RenderAssetUsages::RENDER_WORLD,
                ))),
                MeshMaterial3d(chunk_material.0.clone()),
                Transform::from_translation(chunk_pos),
            ))
            .id();
        chunk_map.0.insert(coord, entity);
    }
}
