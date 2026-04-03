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

/// ブロック座標に対応するブロックを取得 (チャンクをまたぐ場合は隣接チャンクを参照)
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
            BlockType::Stone
        }
    }
}

/// シンプルな可視面カリングメッシュを生成
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

                let block_color = block.base_color();
                let local = IVec3::new(lx as i32, ly as i32, lz as i32);
                let offset = Vec3::new(lx as f32, ly as f32, lz as f32);

                for (dir, face_verts, normal) in &FACES {
                    let neighbor_local = local + *dir;
                    let neighbor = get_block_at(chunk, chunk_store, chunk_coord, neighbor_local);

                    if !neighbor.is_transparent() {
                        continue;
                    }

                    let base_idx = positions.len() as u32;
                    for vert in face_verts {
                        positions.push([
                            offset.x + vert[0],
                            offset.y + vert[1],
                            offset.z + vert[2],
                        ]);
                        normals.push(*normal);
                        uvs.push([0.0, 0.0]);
                        colors.push(block_color);
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
