use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;

use crate::block::BlockType;
use crate::chunk::{ChunkData, ChunkDataStore, CHUNK_SIZE};

const FACES: [(IVec3, [[f32; 3]; 4], [f32; 3]); 6] = [
    // +X
    (
        IVec3::X,
        [
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
            [1.0, 0.0, 1.0],
        ],
        [1.0, 0.0, 0.0],
    ),
    // -X
    (
        IVec3::NEG_X,
        [
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ],
        [-1.0, 0.0, 0.0],
    ),
    // +Y (top)
    (
        IVec3::Y,
        [
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 0.0],
        ],
        [0.0, 1.0, 0.0],
    ),
    // -Y (bottom)
    (
        IVec3::NEG_Y,
        [
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
        ],
        [0.0, -1.0, 0.0],
    ),
    // +Z
    (
        IVec3::Z,
        [
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
        [0.0, 0.0, 1.0],
    ),
    // -Z
    (
        IVec3::NEG_Z,
        [
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
        ],
        [0.0, 0.0, -1.0],
    ),
];

fn get_block_at(
    chunk: &ChunkData,
    chunk_store: &ChunkDataStore,
    chunk_coord: IVec3,
    local: IVec3,
) -> BlockType {
    if local.x >= 0
        && local.y >= 0
        && local.z >= 0
        && local.x < CHUNK_SIZE as i32
        && local.y < CHUNK_SIZE as i32
        && local.z < CHUNK_SIZE as i32
    {
        chunk.get(local.x as usize, local.y as usize, local.z as usize)
    } else {
        let neighbor_coord = chunk_coord
            + IVec3::new(
                local.x.div_euclid(CHUNK_SIZE as i32),
                local.y.div_euclid(CHUNK_SIZE as i32),
                local.z.div_euclid(CHUNK_SIZE as i32),
            );
        let neighbor_local = IVec3::new(
            local.x.rem_euclid(CHUNK_SIZE as i32),
            local.y.rem_euclid(CHUNK_SIZE as i32),
            local.z.rem_euclid(CHUNK_SIZE as i32),
        );
        if let Some(neighbor) = chunk_store.0.get(&neighbor_coord) {
            neighbor.get(
                neighbor_local.x as usize,
                neighbor_local.y as usize,
                neighbor_local.z as usize,
            )
        } else {
            // 隣接チャンクが未ロードの場合は Air とみなして面を描画する
            // (Stone とみなすとチャンク境界の面が不当に非表示になる)
            BlockType::Air
        }
    }
}

/// terrain.png (256x256, 16x16 タイル) のタイル位置を返す (col, row)
fn block_tile(block: BlockType, dir: IVec3) -> (u32, u32) {
    match block {
        BlockType::Grass => {
            if dir == IVec3::Y {
                (0, 0) // 草の上面 (グレースケール→緑で着色)
            } else if dir == IVec3::NEG_Y {
                (2, 0) // 下面は土
            } else {
                (3, 0) // 側面: 草+土
            }
        }
        BlockType::Stone => (1, 0),
        BlockType::Dirt => (2, 0),
        BlockType::Sand => (2, 1),
        BlockType::Cobblestone => (0, 1),
        BlockType::Bedrock => (1, 1),
        BlockType::Wood => {
            if dir.y != 0 { (5, 1) } else { (4, 1) }
        }
        BlockType::Planks => (4, 0),
        BlockType::Glass => (1, 3),
        BlockType::Leaves => (4, 3), // グレースケール→緑で着色
        BlockType::Air => (0, 0),
    }
}

/// タイル (col, row) の UV 座標を返す [v0, v1, v2, v3]
/// v0=左下, v1=左上, v2=右上, v3=右下
fn tile_uv(col: u32, row: u32) -> [[f32; 2]; 4] {
    const S: f32 = 1.0 / 16.0;
    const EPS: f32 = 0.5 / 256.0; // テクスチャ滲み防止
    let u0 = col as f32 * S + EPS;
    let u1 = (col + 1) as f32 * S - EPS;
    let v0 = row as f32 * S + EPS;
    let v1 = (row + 1) as f32 * S - EPS;
    [[u0, v1], [u0, v0], [u1, v0], [u1, v1]]
}

/// 面の頂点カラー: 方向による明度 + バイオーム着色
fn face_color(block: BlockType, dir: IVec3) -> [f32; 4] {
    let brightness = if dir == IVec3::Y {
        1.0_f32
    } else if dir == IVec3::NEG_Y {
        0.5
    } else if dir.x != 0 {
        0.7
    } else {
        0.85
    };

    match (block, dir) {
        // 草の上面: 緑に着色
        (BlockType::Grass, d) if d == IVec3::Y => [0.55, 0.9, 0.3, 1.0],
        // 草の側面: 上部の草部分を少しだけ緑に
        (BlockType::Grass, d) if d.y == 0 => [brightness * 0.85, brightness, brightness * 0.7, 1.0],
        // 葉: 緑に着色 (グレースケールテクスチャを緑に)
        (BlockType::Leaves, _) => [0.4 * brightness, 0.8 * brightness, 0.2 * brightness, 1.0],
        // それ以外: 面方向の明度のみ
        _ => [brightness, brightness, brightness, 1.0],
    }
}

pub fn build_chunk_mesh(
    chunk: &ChunkData,
    chunk_store: &ChunkDataStore,
    chunk_coord: IVec3,
) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for lx in 0..CHUNK_SIZE {
        for ly in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let block = chunk.get(lx, ly, lz);
                if block.is_air() {
                    continue;
                }

                let local = IVec3::new(lx as i32, ly as i32, lz as i32);
                let offset = Vec3::new(lx as f32, ly as f32, lz as f32);

                for (dir, face_verts, normal) in &FACES {
                    let neighbor_local = local + *dir;
                    let neighbor = get_block_at(chunk, chunk_store, chunk_coord, neighbor_local);

                    if !neighbor.is_transparent() {
                        continue;
                    }

                    let (col, row) = block_tile(block, *dir);
                    let face_uvs = tile_uv(col, row);
                    let color = face_color(block, *dir);

                    let base_idx = positions.len() as u32;
                    for (i, vert) in face_verts.iter().enumerate() {
                        positions.push([
                            offset.x + vert[0],
                            offset.y + vert[1],
                            offset.z + vert[2],
                        ]);
                        normals.push(*normal);
                        uvs.push(face_uvs[i]);
                        colors.push(color);
                    }
                    indices.extend_from_slice(&[
                        base_idx,
                        base_idx + 1,
                        base_idx + 2,
                        base_idx,
                        base_idx + 2,
                        base_idx + 3,
                    ]);
                }
            }
        }
    }

    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::Float32x2(uvs),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        VertexAttributeValues::Float32x4(colors),
    );
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
