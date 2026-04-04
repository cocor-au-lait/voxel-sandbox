use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use super::{Player, PlayerCamera, PlayerOnGround, PlayerVelocity};

pub fn spawn_player(mut commands: Commands) {
    // プレイヤー本体 (足元位置)
    commands.spawn((
        Player,
        PlayerVelocity::default(),
        PlayerOnGround::default(),
        Transform::from_xyz(16.0, 68.0, 16.0),
    ));

    // カメラは独立エンティティ (GlobalTransform の遅延を避けるため親子関係を使わない)
    // tonemapping_luts feature なしでは TonyMcMapface が使えないため None に設定
    commands.spawn((
        PlayerCamera::default(),
        Camera3d::default(),
        Tonemapping::None,
        Transform::from_xyz(16.0, 69.65, 16.0),
    ));
}

/// マウス入力でカメラの yaw/pitch を更新、カーソルロック管理
pub fn handle_mouse_look(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut camera_q: Query<&mut PlayerCamera>,
) {
    let Ok(mut cursor) = cursor_q.single_mut() else {
        return;
    };

    if cursor.grab_mode == CursorGrabMode::None {
        // 未ロック: クリックでロック
        if mouse.just_pressed(MouseButton::Left) || mouse.just_pressed(MouseButton::Right) {
            cursor.grab_mode = CursorGrabMode::Locked;
            cursor.visible = false;
        }
        mouse_motion.clear();
        return;
    }

    // ロック中: ESC で解除
    if keyboard.just_pressed(KeyCode::Escape) {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }

    let Ok(mut cam) = camera_q.single_mut() else {
        return;
    };

    for event in mouse_motion.read() {
        cam.yaw -= event.delta.x * cam.sensitivity;
        cam.pitch = (cam.pitch - event.delta.y * cam.sensitivity)
            .clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
    }
}

/// カメラ Transform をプレイヤー位置に同期 (衝突解決後に実行)
pub fn sync_camera_to_player(
    player_q: Query<&Transform, With<Player>>,
    mut camera_q: Query<(&mut Transform, &PlayerCamera), Without<Player>>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let Ok((mut cam_tf, cam)) = camera_q.single_mut() else {
        return;
    };
    cam_tf.translation = player_tf.translation + Vec3::new(0.0, 1.65, 0.0);
    cam_tf.rotation = Quat::from_euler(EulerRot::YXZ, cam.yaw, cam.pitch, 0.0);
}
