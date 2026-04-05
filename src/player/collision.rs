use bevy::prelude::*;
use crate::block::BlockType;
use crate::chunk::{ChunkDataStore, CHUNK_SIZE_I};
use super::{Player, PlayerOnGround, PlayerVelocity};

const HALF_WIDTH: f32 = 0.3;
const HEIGHT: f32 = 1.8;

fn block_at(world_pos: IVec3, store: &ChunkDataStore) -> BlockType {
    let c = IVec3::new(
        world_pos.x.div_euclid(CHUNK_SIZE_I),
        world_pos.y.div_euclid(CHUNK_SIZE_I),
        world_pos.z.div_euclid(CHUNK_SIZE_I),
    );
    let l = IVec3::new(
        world_pos.x.rem_euclid(CHUNK_SIZE_I),
        world_pos.y.rem_euclid(CHUNK_SIZE_I),
        world_pos.z.rem_euclid(CHUNK_SIZE_I),
    );
    store
        .0
        .get(&c)
        .map(|chunk| chunk.get(l.x as usize, l.y as usize, l.z as usize))
        .unwrap_or(BlockType::Air)
}

fn is_solid(world_pos: IVec3, store: &ChunkDataStore) -> bool {
    !matches!(
        block_at(world_pos, store),
        BlockType::Air | BlockType::Leaves | BlockType::Glass
    )
}

pub fn apply_velocity_with_collision(
    time: Res<Time>,
    store: Res<ChunkDataStore>,
    mut q: Query<(&mut Transform, &mut PlayerVelocity, &mut PlayerOnGround), With<Player>>,
) {
    let Ok((mut tf, mut vel, mut on_ground)) = q.single_mut() else {
        return;
    };
    let dt = time.delta_secs().min(0.05);
    let mut pos = tf.translation;
    on_ground.0 = false;

    // Y 軸 (重力 → 着地/頭ぶつけ判定)
    pos.y += vel.0.y * dt;
    resolve_y(&mut pos, &mut vel.0, &mut on_ground.0, &store);

    // X 軸
    pos.x += vel.0.x * dt;
    resolve_x(&mut pos, &mut vel.0, &store);

    // Z 軸
    pos.z += vel.0.z * dt;
    resolve_z(&mut pos, &mut vel.0, &store);

    tf.translation = pos;
}

fn resolve_y(pos: &mut Vec3, vel: &mut Vec3, on_ground: &mut bool, store: &ChunkDataStore) {
    let bx_min = (pos.x - HALF_WIDTH + 0.001).floor() as i32;
    let bx_max = (pos.x + HALF_WIDTH - 0.001).floor() as i32;
    let bz_min = (pos.z - HALF_WIDTH + 0.001).floor() as i32;
    let bz_max = (pos.z + HALF_WIDTH - 0.001).floor() as i32;

    if vel.y <= 0.0 {
        // 下方向: 足元ブロックをチェック
        let by = (pos.y - 0.001).floor() as i32;
        for bx in bx_min..=bx_max {
            for bz in bz_min..=bz_max {
                if is_solid(IVec3::new(bx, by, bz), store) {
                    let top = (by + 1) as f32;
                    if top > pos.y {
                        pos.y = top;
                        vel.y = 0.0;
                        *on_ground = true;
                    }
                }
            }
        }
    } else {
        // 上方向: 頭上ブロックをチェック
        let by = (pos.y + HEIGHT + 0.001).floor() as i32;
        for bx in bx_min..=bx_max {
            for bz in bz_min..=bz_max {
                if is_solid(IVec3::new(bx, by, bz), store) {
                    let bottom = by as f32;
                    let new_y = bottom - HEIGHT;
                    if new_y < pos.y {
                        pos.y = new_y;
                        vel.y = 0.0;
                    }
                }
            }
        }
    }
}

fn resolve_x(pos: &mut Vec3, vel: &mut Vec3, store: &ChunkDataStore) {
    let by_min = pos.y.floor() as i32;
    let by_max = (pos.y + HEIGHT - 0.001).floor() as i32;
    let bz_min = (pos.z - HALF_WIDTH + 0.001).floor() as i32;
    let bz_max = (pos.z + HALF_WIDTH - 0.001).floor() as i32;

    if vel.x > 0.0 {
        let bx = (pos.x + HALF_WIDTH + 0.001).floor() as i32;
        for by in by_min..=by_max {
            for bz in bz_min..=bz_max {
                if is_solid(IVec3::new(bx, by, bz), store) {
                    let new_x = bx as f32 - HALF_WIDTH;
                    if new_x < pos.x {
                        pos.x = new_x;
                        vel.x = 0.0;
                    }
                }
            }
        }
    } else if vel.x < 0.0 {
        let bx = (pos.x - HALF_WIDTH - 0.001).floor() as i32;
        for by in by_min..=by_max {
            for bz in bz_min..=bz_max {
                if is_solid(IVec3::new(bx, by, bz), store) {
                    let new_x = (bx + 1) as f32 + HALF_WIDTH;
                    if new_x > pos.x {
                        pos.x = new_x;
                        vel.x = 0.0;
                    }
                }
            }
        }
    }
}

fn resolve_z(pos: &mut Vec3, vel: &mut Vec3, store: &ChunkDataStore) {
    let by_min = pos.y.floor() as i32;
    let by_max = (pos.y + HEIGHT - 0.001).floor() as i32;
    let bx_min = (pos.x - HALF_WIDTH + 0.001).floor() as i32;
    let bx_max = (pos.x + HALF_WIDTH - 0.001).floor() as i32;

    if vel.z > 0.0 {
        let bz = (pos.z + HALF_WIDTH + 0.001).floor() as i32;
        for by in by_min..=by_max {
            for bx in bx_min..=bx_max {
                if is_solid(IVec3::new(bx, by, bz), store) {
                    let new_z = bz as f32 - HALF_WIDTH;
                    if new_z < pos.z {
                        pos.z = new_z;
                        vel.z = 0.0;
                    }
                }
            }
        }
    } else if vel.z < 0.0 {
        let bz = (pos.z - HALF_WIDTH - 0.001).floor() as i32;
        for by in by_min..=by_max {
            for bx in bx_min..=bx_max {
                if is_solid(IVec3::new(bx, by, bz), store) {
                    let new_z = (bz + 1) as f32 + HALF_WIDTH;
                    if new_z > pos.z {
                        pos.z = new_z;
                        vel.z = 0.0;
                    }
                }
            }
        }
    }
}
