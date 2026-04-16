use bevy::prelude::*;
use std::collections::HashSet;

use crate::block::BlockType;
use crate::chunk::{ChunkDataStore, CHUNK_SIZE_I};
use crate::meshing::MeshingQueue;

/// 落下チェックが必要な砂ブロックのワールド座標セット
#[derive(Resource, Default)]
pub struct FallingSandQueue(pub HashSet<IVec3>);

/// 砂物理の更新タイマー
#[derive(Resource)]
pub struct SandPhysicsTimer(Timer);

impl Default for SandPhysicsTimer {
    fn default() -> Self {
        // 1ティック = 0.05 秒 (20 Hz) — Minecraft の砂落下速度と同等
        Self(Timer::from_seconds(0.05, TimerMode::Repeating))
    }
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FallingSandQueue>()
            .init_resource::<SandPhysicsTimer>()
            .add_systems(Update, sand_gravity);
    }
}

fn chunk_coords(world_pos: IVec3) -> (IVec3, IVec3) {
    let chunk = IVec3::new(
        world_pos.x.div_euclid(CHUNK_SIZE_I),
        world_pos.y.div_euclid(CHUNK_SIZE_I),
        world_pos.z.div_euclid(CHUNK_SIZE_I),
    );
    let local = IVec3::new(
        world_pos.x.rem_euclid(CHUNK_SIZE_I),
        world_pos.y.rem_euclid(CHUNK_SIZE_I),
        world_pos.z.rem_euclid(CHUNK_SIZE_I),
    );
    (chunk, local)
}

fn is_chunk_loaded(world_pos: IVec3, store: &ChunkDataStore) -> bool {
    let (chunk, _) = chunk_coords(world_pos);
    store.0.contains_key(&chunk)
}

fn get_block(world_pos: IVec3, store: &ChunkDataStore) -> BlockType {
    let (chunk, local) = chunk_coords(world_pos);
    store
        .0
        .get(&chunk)
        .map(|data| data.get(local.x as usize, local.y as usize, local.z as usize))
        .unwrap_or(BlockType::Air)
}

fn set_block(
    world_pos: IVec3,
    block: BlockType,
    store: &mut ChunkDataStore,
    meshing_queue: &mut MeshingQueue,
) {
    let (chunk, local) = chunk_coords(world_pos);
    if let Some(chunk_data) = store.0.get_mut(&chunk) {
        chunk_data.set(local.x as usize, local.y as usize, local.z as usize, block);
        meshing_queue.reprioritize(chunk);
        if local.x == 0 {
            meshing_queue.reprioritize(chunk - IVec3::X);
        }
        if local.x == CHUNK_SIZE_I - 1 {
            meshing_queue.reprioritize(chunk + IVec3::X);
        }
        if local.y == 0 {
            meshing_queue.reprioritize(chunk - IVec3::Y);
        }
        if local.y == CHUNK_SIZE_I - 1 {
            meshing_queue.reprioritize(chunk + IVec3::Y);
        }
        if local.z == 0 {
            meshing_queue.reprioritize(chunk - IVec3::Z);
        }
        if local.z == CHUNK_SIZE_I - 1 {
            meshing_queue.reprioritize(chunk + IVec3::Z);
        }
    }
}

/// 砂ブロックの重力物理システム
///
/// 毎ティック、キューにある砂ブロックの下が空気なら 1 ブロック落下させる。
/// 落下後は元の位置の上と新しい位置をキューに再登録し、連鎖落下を実現する。
pub fn sand_gravity(
    time: Res<Time>,
    mut timer: ResMut<SandPhysicsTimer>,
    mut sand_queue: ResMut<FallingSandQueue>,
    mut store: ResMut<ChunkDataStore>,
    mut meshing_queue: ResMut<MeshingQueue>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let positions: Vec<IVec3> = sand_queue.0.drain().collect();
    let mut next_positions: HashSet<IVec3> = HashSet::new();

    for pos in positions {
        // チャンクがロードされていなければスキップ
        if !is_chunk_loaded(pos, &store) {
            continue;
        }

        if get_block(pos, &store) != BlockType::Sand {
            continue;
        }

        let below = pos - IVec3::Y;

        // 下のチャンクが未ロードなら落下させない（消滅防止）
        if !is_chunk_loaded(below, &store) {
            continue;
        }

        if get_block(below, &store) == BlockType::Air {
            // 砂を 1 ブロック落下
            set_block(pos, BlockType::Air, &mut store, &mut meshing_queue);
            set_block(below, BlockType::Sand, &mut store, &mut meshing_queue);

            // 元の位置の上にある砂も落下する可能性がある
            next_positions.insert(pos + IVec3::Y);
            // 新しい位置の砂がさらに落下する可能性がある
            next_positions.insert(below);
        }
    }

    sand_queue.0.extend(next_positions);
}
