use bevy::prelude::*;

mod block;
mod chunk;
mod inventory;
mod meshing;
mod persistence;
mod player;
mod rendering;
mod terrain;
mod utils;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Voxel Sandbox".into(),
                        // wasm-server-runner は独自 HTML を生成するため canvas セレクタは不要
                        // 独自 HTML でデプロイする場合は Some("#bevy-canvas".into()) に変更
                        canvas: None,
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins((
            block::BlockPlugin,
            chunk::ChunkPlugin,
            terrain::TerrainPlugin,
            meshing::MeshingPlugin,
            rendering::RenderingPlugin,
            player::PlayerPlugin,
            inventory::InventoryPlugin,
            persistence::PersistencePlugin,
        ))
        .add_systems(Startup, setup_world)
        .run();
}

fn setup_world(mut commands: Commands) {
    // 太陽光
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 10_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.5, 0.0)),
    ));

    // 環境光 (Bevy 0.18 では Component として spawn)
    commands.spawn(AmbientLight {
        color: Color::srgb(0.6, 0.7, 1.0),
        brightness: 300.0,
        affects_lightmapped_meshes: true,
    });
}
