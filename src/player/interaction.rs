use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use crate::block::BlockType;
use crate::chunk::{ChunkDataStore, CHUNK_SIZE_I};
use crate::meshing::MeshingQueue;
use super::{Player, PlayerCamera, SelectedBlock, TargetBlock};

const MAX_REACH: f32 = 6.0;

#[derive(Component)]
pub struct HighlightBlock;

/// カーソルが未ロック時に表示するオーバーレイ
#[derive(Component)]
pub struct ClickToPlayOverlay;


/// ゲーム開始時: ハイライトボックスとクロスヘアを生成
pub fn setup_hud(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 選択ブロックのハイライト
    commands.spawn((
        HighlightBlock,
        Mesh3d(meshes.add(Cuboid::new(1.004, 1.004, 1.004))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, 0.25),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        Transform::IDENTITY,
        Visibility::Hidden,
    ));

    // クロスヘア: 全画面中央コンテナ → 中心点 0x0 → 水平・垂直バーを絶対配置
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|center| {
            center
                .spawn(Node {
                    width: Val::Px(0.0),
                    height: Val::Px(0.0),
                    ..default()
                })
                .with_children(|cross| {
                    // 水平バー
                    cross.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Px(16.0),
                            height: Val::Px(2.0),
                            left: Val::Px(-8.0),
                            top: Val::Px(-1.0),
                            ..default()
                        },
                        BackgroundColor(Color::WHITE),
                    ));
                    // 垂直バー
                    cross.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Px(2.0),
                            height: Val::Px(16.0),
                            left: Val::Px(-1.0),
                            top: Val::Px(-8.0),
                            ..default()
                        },
                        BackgroundColor(Color::WHITE),
                    ));
                });
        });

    // TODO: 操作説明オーバーレイ未実装 (テキスト ContentSize 伝搬の問題で一時削除)

    // クリックで開始オーバーレイ (カーソル未ロック時に表示)
    commands
        .spawn((
            ClickToPlayOverlay,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Click to play\n[ESC] to release cursor"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    ..default()
                },
            ));
        });
}

/// DDA レイキャスト: カーソルロック中のみ実行
pub fn cast_ray(
    camera_q: Query<&Transform, With<PlayerCamera>>,
    cursor_q: Query<&CursorOptions, With<PrimaryWindow>>,
    store: Res<ChunkDataStore>,
    mut target: ResMut<TargetBlock>,
) {
    let Ok(cursor) = cursor_q.single() else {
        return;
    };
    if cursor.grab_mode == CursorGrabMode::None {
        target.pos = None;
        target.normal = None;
        return;
    }

    let Ok(cam_tf) = camera_q.single() else {
        target.pos = None;
        target.normal = None;
        return;
    };

    let origin = cam_tf.translation;
    let dir = cam_tf.forward().as_vec3();

    match dda(origin, dir, MAX_REACH, &store) {
        Some((pos, normal)) => {
            target.pos = Some(pos);
            target.normal = Some(normal);
        }
        None => {
            target.pos = None;
            target.normal = None;
        }
    }
}

/// Amanatides & Woo DDA レイキャスト
fn dda(
    origin: Vec3,
    dir: Vec3,
    max_dist: f32,
    store: &ChunkDataStore,
) -> Option<(IVec3, IVec3)> {
    let dir = dir.normalize();
    let step = IVec3::new(
        if dir.x >= 0.0 { 1 } else { -1 },
        if dir.y >= 0.0 { 1 } else { -1 },
        if dir.z >= 0.0 { 1 } else { -1 },
    );

    let mut block = origin.floor().as_ivec3();

    let next = Vec3::new(
        if dir.x >= 0.0 { (block.x + 1) as f32 } else { block.x as f32 },
        if dir.y >= 0.0 { (block.y + 1) as f32 } else { block.y as f32 },
        if dir.z >= 0.0 { (block.z + 1) as f32 } else { block.z as f32 },
    );

    let mut t_max = Vec3::new(
        if dir.x.abs() > 1e-8 { (next.x - origin.x) / dir.x } else { f32::MAX },
        if dir.y.abs() > 1e-8 { (next.y - origin.y) / dir.y } else { f32::MAX },
        if dir.z.abs() > 1e-8 { (next.z - origin.z) / dir.z } else { f32::MAX },
    );

    let t_delta = Vec3::new(
        if dir.x.abs() > 1e-8 { (step.x as f32).abs() / dir.x.abs() } else { f32::MAX },
        if dir.y.abs() > 1e-8 { (step.y as f32).abs() / dir.y.abs() } else { f32::MAX },
        if dir.z.abs() > 1e-8 { (step.z as f32).abs() / dir.z.abs() } else { f32::MAX },
    );

    let mut normal = IVec3::ZERO;

    loop {
        // 次のグリッド境界を選択してから境界チェック
        if t_max.x < t_max.y && t_max.x < t_max.z {
            if t_max.x > max_dist {
                break;
            }
            block.x += step.x;
            normal = IVec3::new(-step.x, 0, 0);
            t_max.x += t_delta.x;
        } else if t_max.y < t_max.z {
            if t_max.y > max_dist {
                break;
            }
            block.y += step.y;
            normal = IVec3::new(0, -step.y, 0);
            t_max.y += t_delta.y;
        } else {
            if t_max.z > max_dist {
                break;
            }
            block.z += step.z;
            normal = IVec3::new(0, 0, -step.z);
            t_max.z += t_delta.z;
        }

        if is_targetable(block, store) {
            return Some((block, normal));
        }
    }

    None
}

fn is_targetable(world_pos: IVec3, store: &ChunkDataStore) -> bool {
    let block = get_block(world_pos, store);
    !block.is_air()
}

fn get_block(world_pos: IVec3, store: &ChunkDataStore) -> BlockType {
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

/// 左クリック: ブロック破壊 / 右クリック: ブロック設置
pub fn handle_block_input(
    mouse: Res<ButtonInput<MouseButton>>,
    cursor_q: Query<&CursorOptions, With<PrimaryWindow>>,
    target: Res<TargetBlock>,
    selected: Res<SelectedBlock>,
    player_q: Query<&Transform, With<Player>>,
    mut store: ResMut<ChunkDataStore>,
    mut queue: ResMut<MeshingQueue>,
) {
    let Ok(cursor) = cursor_q.single() else {
        return;
    };
    if cursor.grab_mode == CursorGrabMode::None {
        return;
    }

    let Some(pos) = target.pos else {
        return;
    };

    if mouse.just_pressed(MouseButton::Left) {
        set_block(pos, BlockType::Air, &mut store, &mut queue);
    }

    if mouse.just_pressed(MouseButton::Right) {
        let Some(normal) = target.normal else {
            return;
        };
        let place_pos = pos + normal;

        // プレイヤー AABB と重ならないことを確認
        let Ok(player_tf) = player_q.single() else {
            return;
        };
        if !overlaps_player(place_pos, player_tf.translation) {
            set_block(place_pos, selected.0, &mut store, &mut queue);
        }
    }
}

fn overlaps_player(block: IVec3, feet: Vec3) -> bool {
    let bmin = block.as_vec3();
    let bmax = bmin + Vec3::ONE;
    let pmin = feet + Vec3::new(-0.3, 0.0, -0.3);
    let pmax = feet + Vec3::new(0.3, 1.8, 0.3);
    bmax.x > pmin.x
        && bmin.x < pmax.x
        && bmax.y > pmin.y
        && bmin.y < pmax.y
        && bmax.z > pmin.z
        && bmin.z < pmax.z
}

fn set_block(
    world_pos: IVec3,
    block: BlockType,
    store: &mut ChunkDataStore,
    queue: &mut MeshingQueue,
) {
    let chunk = IVec3::new(
        world_pos.x.div_euclid(CHUNK_SIZE_I),
        world_pos.y.div_euclid(CHUNK_SIZE_I),
        world_pos.z.div_euclid(CHUNK_SIZE_I),
    );
    let local = IVec3::new(
        world_pos.x.rem_euclid(CHUNK_SIZE_I),
        world_pos.y.rem_euclid(CHUNK_SIZE_I),
        world_pos.z.rem_euclid(CHUNK_SIZE_I),
    );
    if let Some(chunk_data) = store.0.get_mut(&chunk) {
        chunk_data.set(local.x as usize, local.y as usize, local.z as usize, block);
        queue.0.push_front(chunk);
        // 境界ブロックなら隣接チャンクも再メッシュ
        if local.x == 0 {
            queue.0.push_front(chunk - IVec3::X);
        }
        if local.x == CHUNK_SIZE_I - 1 {
            queue.0.push_front(chunk + IVec3::X);
        }
        if local.y == 0 {
            queue.0.push_front(chunk - IVec3::Y);
        }
        if local.y == CHUNK_SIZE_I - 1 {
            queue.0.push_front(chunk + IVec3::Y);
        }
        if local.z == 0 {
            queue.0.push_front(chunk - IVec3::Z);
        }
        if local.z == CHUNK_SIZE_I - 1 {
            queue.0.push_front(chunk + IVec3::Z);
        }
    }
}

/// ハイライトボックスをターゲットブロック位置に移動
/// カーソルロック状態に応じてオーバーレイを表示/非表示
pub fn update_overlay(
    cursor_q: Query<&CursorOptions, With<PrimaryWindow>>,
    mut overlay_q: Query<&mut Visibility, With<ClickToPlayOverlay>>,
) {
    let Ok(cursor) = cursor_q.single() else {
        return;
    };
    let Ok(mut vis) = overlay_q.single_mut() else {
        return;
    };
    *vis = if cursor.grab_mode == CursorGrabMode::None {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

pub fn update_highlight(
    target: Res<TargetBlock>,
    mut q: Query<(&mut Transform, &mut Visibility), With<HighlightBlock>>,
) {
    let Ok((mut tf, mut vis)) = q.single_mut() else {
        return;
    };
    match target.pos {
        Some(pos) => {
            tf.translation = pos.as_vec3() + Vec3::splat(0.5);
            *vis = Visibility::Visible;
        }
        None => {
            *vis = Visibility::Hidden;
        }
    }
}
