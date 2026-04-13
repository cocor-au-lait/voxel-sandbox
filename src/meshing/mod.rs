use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};

use crate::chunk::chunk_manager;
use crate::chunk::{ChunkCoord, ChunkEntity, ChunkMap};

pub mod simple_mesher;

pub use simple_mesher::build_chunk_mesh;

/// メッシュ生成待ちキュー（重複排除付き）
///
/// 同一チャンクを複数回キューに積まないよう HashSet で管理する。
/// 隣接チャンクのロード時に再メッシュ要求が大量に発生しても
/// キューが膨れ上がらないようにするための対策。
///
/// # 設計
/// `in_queue` を "処理すべき座標の正規セット" として扱う。
/// - `push_back` / `push_front`: in_queue に未登録なら追加
/// - `reprioritize`: 必ず先頭に追加（古いエントリは pop 時にスキップ）
/// - `pop_front`: in_queue から除去できたエントリのみ返す
#[derive(Resource, Default)]
pub struct MeshingQueue {
    queue: VecDeque<IVec3>,
    in_queue: HashSet<IVec3>,
}

impl MeshingQueue {
    /// キュー末尾に追加（すでに登録済みなら無視）
    pub fn push_back(&mut self, coord: IVec3) {
        if self.in_queue.insert(coord) {
            self.queue.push_back(coord);
        }
    }

    /// キュー先頭に追加（すでに登録済みなら無視）
    pub fn push_front(&mut self, coord: IVec3) {
        if self.in_queue.insert(coord) {
            self.queue.push_front(coord);
        }
    }

    /// キュー先頭に追加して優先処理させる（ブロック操作時用）
    ///
    /// すでにキュー内にある場合も無条件に先頭へ再追加する。
    /// 古い位置のエントリは pop_front 時に自動スキップされる。
    pub fn reprioritize(&mut self, coord: IVec3) {
        self.queue.push_front(coord);
        self.in_queue.insert(coord);
    }

    /// 先頭から有効なエントリを取り出す。古い重複エントリは自動スキップ。
    fn pop_front(&mut self) -> Option<IVec3> {
        loop {
            let coord = self.queue.pop_front()?;
            if self.in_queue.remove(&coord) {
                return Some(coord);
            }
            // reprioritize によって先頭に移動済みの古いエントリ → スキップ
        }
    }

    fn is_empty(&self) -> bool {
        self.in_queue.is_empty()
    }
}

pub struct MeshingPlugin;

impl Plugin for MeshingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeshingQueue>()
            .add_systems(
                Update,
                // sync_chunk_entities (デスポーン) の後にメッシュ化＆スポーンを実行する
                process_meshing_queue.after(chunk_manager::sync_chunk_entities),
            );
    }
}

/// 毎フレーム最大 MESH_BUDGET 個のチャンクをメッシュ化・スポーン
///
/// エンティティのスポーンもここで行う。正しいメッシュが完成してから
/// スポーンすることで、空メッシュが一瞬表示される問題を防ぐ。
/// メッシュ生成は 1〜3ms/チャンク。16ms フレームで 8 個処理しても約 10ms 以内。
const MESH_BUDGET: usize = 8;

fn process_meshing_queue(
    mut commands: Commands,
    mut queue: ResMut<MeshingQueue>,
    chunk_store: Res<crate::chunk::ChunkDataStore>,
    mut chunk_map: ResMut<ChunkMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_material: Res<crate::rendering::ChunkMaterial>,
    mut mesh_query: Query<&mut Mesh3d>,
) {
    let mut processed = 0;
    while !queue.is_empty() {
        if processed >= MESH_BUDGET {
            break;
        }
        let coord = queue.pop_front().unwrap();

        let Some(chunk_data) = chunk_store.0.get(&coord) else {
            continue;
        };

        let mesh = build_chunk_mesh(chunk_data, &chunk_store, coord);
        let mesh_handle = meshes.add(mesh);

        if let Some(&entity) = chunk_map.0.get(&coord) {
            // エンティティ既存: メッシュを更新（隣接チャンク変化による再メッシュ）
            if let Ok(mut mesh3d) = mesh_query.get_mut(entity) {
                mesh3d.0 = mesh_handle;
            }
        } else {
            // エンティティ未存在: 正しいメッシュと同時にスポーン（空メッシュを挟まない）
            let chunk_pos = ChunkCoord(coord).world_origin();
            let entity = commands
                .spawn((
                    ChunkEntity,
                    ChunkCoord(coord),
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(chunk_material.0.clone()),
                    Transform::from_translation(chunk_pos),
                ))
                .id();
            chunk_map.0.insert(coord, entity);
        }

        processed += 1;
    }
}
