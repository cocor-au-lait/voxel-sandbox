use bevy::prelude::*;

pub mod simple_mesher;

pub use simple_mesher::build_chunk_mesh;

/// メッシュ生成待ちキュー
#[derive(Resource, Default)]
pub struct MeshingQueue(pub std::collections::VecDeque<IVec3>);

pub struct MeshingPlugin;

impl Plugin for MeshingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeshingQueue>()
            .add_systems(Update, process_meshing_queue);
    }
}

/// 毎フレーム最大 MESH_BUDGET 個のチャンクをメッシュ化
const MESH_BUDGET: usize = 2;

fn process_meshing_queue(
    mut queue: ResMut<MeshingQueue>,
    chunk_store: Res<crate::chunk::ChunkDataStore>,
    chunk_map: Res<crate::chunk::ChunkMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_query: Query<&mut Mesh3d>,
) {
    let mut processed = 0;
    while let Some(coord) = queue.0.front().copied() {
        if processed >= MESH_BUDGET {
            break;
        }
        queue.0.pop_front();

        let Some(chunk_data) = chunk_store.0.get(&coord) else {
            continue;
        };
        let Some(&entity) = chunk_map.0.get(&coord) else {
            continue;
        };

        let mesh = build_chunk_mesh(chunk_data, &chunk_store, coord);
        let mesh_handle = meshes.add(mesh);

        if let Ok(mut mesh3d) = mesh_query.get_mut(entity) {
            mesh3d.0 = mesh_handle;
        }

        processed += 1;
    }
}
