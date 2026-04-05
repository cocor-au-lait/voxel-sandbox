use bevy::prelude::*;
use super::{Player, PlayerCamera, PlayerOnGround, PlayerVelocity};

const GRAVITY: f32 = -24.0;
const MOVE_SPEED: f32 = 6.0;
const JUMP_VELOCITY: f32 = 8.0;
const TERMINAL_VELOCITY: f32 = -50.0;

pub fn apply_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_q: Query<&PlayerCamera>,
    mut player_q: Query<(&mut PlayerVelocity, &PlayerOnGround), With<Player>>,
) {
    let Ok(cam) = camera_q.single() else {
        return;
    };
    let Ok((mut vel, on_ground)) = player_q.single_mut() else {
        return;
    };

    let dt = time.delta_secs().min(0.05);

    // 水平移動 (カメラの yaw から XZ 平面方向を計算)
    let forward = Vec3::new(-cam.yaw.sin(), 0.0, -cam.yaw.cos());
    let right = Vec3::new(cam.yaw.cos(), 0.0, -cam.yaw.sin());

    let mut horizontal = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        horizontal += forward;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        horizontal -= forward;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        horizontal -= right;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        horizontal += right;
    }

    if horizontal.length_squared() > 0.0 {
        horizontal = horizontal.normalize() * MOVE_SPEED;
    }
    vel.0.x = horizontal.x;
    vel.0.z = horizontal.z;

    // 重力
    vel.0.y = (vel.0.y + GRAVITY * dt).max(TERMINAL_VELOCITY);

    // ジャンプ (地面にいるときのみ)
    if on_ground.0 && keyboard.just_pressed(KeyCode::Space) {
        vel.0.y = JUMP_VELOCITY;
    }
}
