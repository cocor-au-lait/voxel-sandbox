use bevy::{input::mouse::MouseWheel, prelude::*};
use crate::block::BlockType;
use crate::player::SelectedBlock;
use crate::rendering::{TerrainTexture, setup_chunk_material};

/// ホットバーに並ぶブロック (左から順に 1-9 キーに対応)
pub const HOTBAR_BLOCKS: [BlockType; 9] = [
    BlockType::Stone,
    BlockType::Dirt,
    BlockType::Grass,
    BlockType::Sand,
    BlockType::Wood,
    BlockType::Cobblestone,
    BlockType::Planks,
    BlockType::Glass,
    BlockType::Leaves,
];

/// terrain.png 内のタイル位置 (col, row) と色チント
fn hotbar_tile(block: BlockType) -> ((f32, f32), Color) {
    match block {
        BlockType::Stone      => ((1.0, 0.0), Color::WHITE),
        BlockType::Dirt       => ((2.0, 0.0), Color::WHITE),
        BlockType::Grass      => ((0.0, 0.0), Color::srgb(0.55, 0.9, 0.3)),
        BlockType::Sand       => ((2.0, 1.0), Color::WHITE),
        BlockType::Wood       => ((5.0, 1.0), Color::WHITE),
        BlockType::Cobblestone=> ((0.0, 1.0), Color::WHITE),
        BlockType::Planks     => ((4.0, 0.0), Color::WHITE),
        BlockType::Glass      => ((1.0, 3.0), Color::WHITE),
        BlockType::Leaves     => ((4.0, 3.0), Color::srgb(0.4, 0.8, 0.2)),
        _                     => ((0.0, 0.0), Color::WHITE),
    }
}

/// terrain.png の 1 タイルサイズ (px)
const TILE_PX: f32 = 16.0;

/// 現在選択中のホットバースロット (0-8)
#[derive(Resource, Default)]
pub struct Hotbar {
    pub selected: usize,
}

/// ホットバーの各スロットにつくマーカー
#[derive(Component)]
pub struct HotbarSlot(pub usize);

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Hotbar>()
            .add_systems(Startup, setup_hotbar.after(setup_chunk_material))
            .add_systems(
                Update,
                (handle_hotbar_input, sync_hotbar_ui, sync_selected_block).chain(),
            );
    }
}

fn setup_hotbar(mut commands: Commands, terrain_tex: Res<TerrainTexture>) {
    // 全画面コンテナ: 縦方向末尾 (下部) × 横方向中央
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::End,
            align_items: AlignItems::Center,
            padding: UiRect::bottom(Val::Px(10.0)),
            ..default()
        })
        .with_children(|root| {
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(3.0),
                ..default()
            })
            .with_children(|row| {
                for i in 0..9 {
                    let border_color = if i == 0 { Color::WHITE } else { Color::srgba(0.5, 0.5, 0.5, 1.0) };
                    let ((col, tile_row), tint) = hotbar_tile(HOTBAR_BLOCKS[i]);
                    let rect = Rect::new(
                        col * TILE_PX,
                        tile_row * TILE_PX,
                        (col + 1.0) * TILE_PX,
                        (tile_row + 1.0) * TILE_PX,
                    );

                    row.spawn((
                        HotbarSlot(i),
                        Node {
                            width: Val::Px(44.0),
                            height: Val::Px(44.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                        BorderColor::all(border_color),
                    ))
                    .with_children(|slot| {
                        slot.spawn((
                            Node {
                                width: Val::Px(32.0),
                                height: Val::Px(32.0),
                                ..default()
                            },
                            ImageNode {
                                image: terrain_tex.0.clone(),
                                rect: Some(rect),
                                color: tint,
                                image_mode: NodeImageMode::Stretch,
                                ..default()
                            },
                        ));
                    });
                }
            });
        });
}

/// 数字キー 1-9 またはスクロールホイールでスロットを切り替える
fn handle_hotbar_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut scroll: MessageReader<MouseWheel>,
    mut hotbar: ResMut<Hotbar>,
) {
    let keys = [
        KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3,
        KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6,
        KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9,
    ];
    for (i, &key) in keys.iter().enumerate() {
        if keyboard.just_pressed(key) {
            hotbar.selected = i;
        }
    }

    for event in scroll.read() {
        // scroll up (y > 0) → 右へ, scroll down (y < 0) → 左へ
        let delta: i32 = if event.y > 0.0 { 1 } else { -1 };
        hotbar.selected = ((hotbar.selected as i32 + delta).rem_euclid(9)) as usize;
    }
}

/// ホットバー UI の選択ハイライトを更新する
fn sync_hotbar_ui(
    hotbar: Res<Hotbar>,
    mut slot_q: Query<(&HotbarSlot, &mut BorderColor)>,
) {
    if !hotbar.is_changed() {
        return;
    }
    for (slot, mut border) in slot_q.iter_mut() {
        let c = if slot.0 == hotbar.selected {
            Color::WHITE
        } else {
            Color::srgba(0.5, 0.5, 0.5, 1.0)
        };
        *border = BorderColor::all(c);
    }
}

/// SelectedBlock リソースをホットバー選択と同期する
fn sync_selected_block(hotbar: Res<Hotbar>, mut selected: ResMut<SelectedBlock>) {
    if hotbar.is_changed() {
        selected.0 = HOTBAR_BLOCKS[hotbar.selected];
    }
}
