use bevy::prelude::*;
use noise::{Fbm, NoiseFn, Perlin};

use crate::block::BlockType;
use crate::chunk::{ChunkCoord, ChunkData, CHUNK_SIZE, CHUNK_SIZE_I};

#[derive(Clone, Copy, PartialEq)]
enum Biome {
    Plains,
    Desert,
    Forest,
}

/// プロシージャル地形生成器
#[derive(Resource)]
pub struct TerrainGenerator {
    height_noise: Fbm<Perlin>,
    biome_noise: Fbm<Perlin>,
    cave_noise: Fbm<Perlin>,
    seed: u32,
}

impl TerrainGenerator {
    pub fn new(seed: u32) -> Self {
        let mut height_noise = Fbm::<Perlin>::new(seed);
        height_noise.octaves = 6;
        height_noise.frequency = 1.0;
        height_noise.lacunarity = 2.0;
        height_noise.persistence = 0.5;

        let mut biome_noise = Fbm::<Perlin>::new(seed + 1);
        biome_noise.octaves = 4;
        biome_noise.frequency = 1.0;
        biome_noise.lacunarity = 2.0;
        biome_noise.persistence = 0.5;

        let mut cave_noise = Fbm::<Perlin>::new(seed + 2);
        cave_noise.octaves = 4;
        cave_noise.frequency = 1.0;
        cave_noise.lacunarity = 2.0;
        cave_noise.persistence = 0.5;

        Self {
            height_noise,
            biome_noise,
            cave_noise,
            seed,
        }
    }

    /// ワールド座標 (wx, wz) の地表高さを返す（ブロック座標）
    fn surface_height(&self, wx: i32, wz: i32) -> i32 {
        let val = self
            .height_noise
            .get([wx as f64 * 0.004, wz as f64 * 0.004]);
        63 + (val * 30.0) as i32
    }

    /// ワールド座標 (wx, wz) のバイオームを返す
    fn biome(&self, wx: i32, wz: i32) -> Biome {
        let val = self
            .biome_noise
            .get([wx as f64 * 0.002, wz as f64 * 0.002]);
        if val < -0.3 {
            Biome::Desert
        } else if val > 0.3 {
            Biome::Forest
        } else {
            Biome::Plains
        }
    }

    /// 3D ノイズで洞窟かどうか判定
    fn is_cave(&self, wx: i32, wy: i32, wz: i32) -> bool {
        let val = self.cave_noise.get([
            wx as f64 * 0.04,
            wy as f64 * 0.02,
            wz as f64 * 0.04,
        ]);
        val > 0.62 && wy > 5
    }

    /// 指定ワールド座標のブロック種別を決定
    fn block_at(&self, wx: i32, wy: i32, wz: i32, surface: i32, biome: Biome) -> BlockType {
        if wy < 0 {
            BlockType::Bedrock
        } else if wy > surface {
            BlockType::Air
        } else if wy == surface {
            match biome {
                Biome::Desert => BlockType::Sand,
                _ if surface > 85 => BlockType::Stone,
                _ => BlockType::Grass,
            }
        } else if wy > surface - 4 {
            match biome {
                Biome::Desert => BlockType::Sand,
                _ => BlockType::Dirt,
            }
        } else if self.is_cave(wx, wy, wz) {
            BlockType::Air
        } else {
            BlockType::Stone
        }
    }

    /// (wx, wz) に木を生成するか（決定論的ハッシュ）
    fn should_place_tree(&self, wx: i32, wz: i32) -> bool {
        let mut h: u32 = self.seed;
        h ^= (wx as u32).wrapping_mul(0x9e3779b9);
        h ^= (wz as u32).wrapping_mul(0x6c62272e);
        h ^= h >> 16;
        h = h.wrapping_mul(0x45d9f3b);
        h ^= h >> 16;
        h % 50 == 0
    }

    /// チャンク内に木を配置する
    fn place_tree(&self, chunk: &mut ChunkData, lx: usize, surface: i32, lz: usize, base_y: i32) {
        let trunk_height = 4i32;

        // 幹
        for i in 1..=trunk_height {
            let ly = surface + i - base_y;
            if ly >= 0 && ly < CHUNK_SIZE_I {
                chunk.set(lx, ly as usize, lz, BlockType::Wood);
            }
        }

        // 葉（幹頂上を中心に ±2(x/z), -1〜+1(y)）
        let leaf_center_y = surface + trunk_height;
        for dy in -1i32..=1 {
            for dx in -2i32..=2 {
                for dz in -2i32..=2 {
                    // 角を丸める
                    if dx.abs() == 2 && dz.abs() == 2 {
                        continue;
                    }
                    let leaf_lx = lx as i32 + dx;
                    let leaf_lz = lz as i32 + dz;
                    let leaf_ly = leaf_center_y + dy - base_y;

                    if leaf_lx >= 0
                        && leaf_lx < CHUNK_SIZE_I
                        && leaf_lz >= 0
                        && leaf_lz < CHUNK_SIZE_I
                        && leaf_ly >= 0
                        && leaf_ly < CHUNK_SIZE_I
                    {
                        let lx2 = leaf_lx as usize;
                        let ly2 = leaf_ly as usize;
                        let lz2 = leaf_lz as usize;
                        if chunk.get(lx2, ly2, lz2) == BlockType::Air {
                            chunk.set(lx2, ly2, lz2, BlockType::Leaves);
                        }
                    }
                }
            }
        }
    }

    /// チャンクを生成する（chunk_manager から呼ばれる）
    pub fn generate_chunk(&self, coord: &ChunkCoord) -> ChunkData {
        let mut chunk = ChunkData::new_empty();
        let base = coord.0 * CHUNK_SIZE_I;

        // Pass 1: 地形ブロックの配置
        let mut surface_heights = [[0i32; CHUNK_SIZE]; CHUNK_SIZE];
        let mut biomes = [[Biome::Plains; CHUNK_SIZE]; CHUNK_SIZE];

        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let wx = base.x + lx as i32;
                let wz = base.z + lz as i32;
                let surface = self.surface_height(wx, wz);
                let biome = self.biome(wx, wz);
                surface_heights[lx][lz] = surface;
                biomes[lx][lz] = biome;

                for ly in 0..CHUNK_SIZE {
                    let wy = base.y + ly as i32;
                    let block = self.block_at(wx, wy, wz, surface, biome);
                    chunk.set(lx, ly, lz, block);
                }
            }
        }

        // Pass 2: 木の配置（チャンク端 3 ブロックは省略してチャンク内に収める）
        for lx in 3..(CHUNK_SIZE - 3) {
            for lz in 3..(CHUNK_SIZE - 3) {
                if biomes[lx][lz] == Biome::Desert {
                    continue;
                }
                let wx = base.x + lx as i32;
                let wz = base.z + lz as i32;
                if self.should_place_tree(wx, wz) {
                    let surface = surface_heights[lx][lz];
                    self.place_tree(&mut chunk, lx, surface, lz, base.y);
                }
            }
        }

        chunk
    }
}
