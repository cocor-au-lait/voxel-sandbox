use bevy::asset::RenderAssetUsages;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;

/// 全チャンクで共有するテクスチャアトラスマテリアル
#[derive(Resource)]
pub struct ChunkMaterial(pub Handle<StandardMaterial>);

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_chunk_material);
    }
}

static TERRAIN_PNG: &[u8] = include_bytes!("../../assets/textures/terrain.png");

fn setup_chunk_material(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let image = Image::from_buffer(
        TERRAIN_PNG,
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::nearest(),
        RenderAssetUsages::RENDER_WORLD,
    )
    .expect("terrain.png の読み込みに失敗");

    let texture = images.add(image);
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        base_color: Color::WHITE,
        unlit: true,
        alpha_mode: AlphaMode::Mask(0.1),
        ..default()
    });
    commands.insert_resource(ChunkMaterial(material));
}

