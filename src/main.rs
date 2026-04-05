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
        .insert_resource(ClearColor(Color::srgb(0.47, 0.66, 1.0)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Voxel Sandbox".into(),
                        // wasm-server-runner は独自 HTML を生成するため canvas セレクタは不要
                        // 独自 HTML でデプロイする場合は Some("#bevy-canvas".into()) に変更
                        canvas: None,
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
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
        .add_systems(Startup, (setup_world, hide_loading_screen))
        .run();
}

/// WASM: trunk が生成するローディング画面を非表示にする
fn hide_loading_screen() {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        if let Some(el) = document.get_element_by_id("loading") {
            let _ = el.set_attribute("style", "display:none");
        }
    }
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
