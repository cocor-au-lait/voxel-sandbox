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
        meshing_queue.push_back(coord);

        // 隣接チャンクが既にロード済みであれば境界面を更新するために再メッシュする
        for dir in [IVec3::X, IVec3::NEG_X, IVec3::Y, IVec3::NEG_Y, IVec3::Z, IVec3::NEG_Z] {
            let neighbor = coord + dir;
            if chunk_store.0.contains_key(&neighbor) {
                meshing_queue.push_back(neighbor);
            }
        }

        count += 1;
    }
}

/// 毎フレーム: アンロードされたチャンクのEntityを削除
///
/// スポーンは process_meshing_queue が正しいメッシュと同時に行う。
/// これにより空メッシュが一瞬表示される問題を防ぐ。
pub fn sync_chunk_entities(
    mut commands: Commands,
    mut chunk_map: ResMut<ChunkMap>,
    chunk_store: Res<ChunkDataStore>,
) {
    chunk_map.0.retain(|&coord, &mut entity| {
        if chunk_store.0.contains_key(&coord) {
            true
        } else {
            commands.entity(entity).despawn();
            false
        }
    });
}
