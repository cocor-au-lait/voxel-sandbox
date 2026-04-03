use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use super::Player;

#[derive(Component)]
pub struct PlayerCamera {
    pub pitch: f32,
    pub yaw: f32,
    pub sensitivity: f32,
    pub speed: f32,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            pitch: 0.0,
            yaw: 0.0,
            sensitivity: 0.002,
            speed: 20.0,
        }
    }
}

pub fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        PlayerCamera::default(),
        Camera3d::default(),
        Transform::from_xyz(16.0, 80.0, 16.0).looking_at(Vec3::new(48.0, 80.0, 16.0), Vec3::Y),
    ));
}

pub fn fly_camera(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut camera_q: Query<(&mut Transform, &mut PlayerCamera)>,
) {
    let Ok(mut cursor) = cursor_q.single_mut() else {
        return;
    };

    // ESC でカーソルトグル
    if keyboard.just_pressed(KeyCode::Escape) {
        match cursor.grab_mode {
            CursorGrabMode::None => {
                cursor.grab_mode = CursorGrabMode::Locked;
                cursor.visible = false;
            }
            _ => {
                cursor.grab_mode = CursorGrabMode::None;
                cursor.visible = true;
            }
        }
    }

    let Ok((mut tf, mut cam)) = camera_q.single_mut() else {
        return;
    };

    // マウス回転 (カーソルロック時のみ)
    if cursor.grab_mode == CursorGrabMode::Locked {
        for event in mouse_motion.read() {
            cam.yaw -= event.delta.x * cam.sensitivity;
            cam.pitch = (cam.pitch - event.delta.y * cam.sensitivity)
                .clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        }
    } else {
        mouse_motion.clear();
    }

    tf.rotation = Quat::from_euler(EulerRot::YXZ, cam.yaw, cam.pitch, 0.0);

    // キーボード移動
    let forward = tf.forward();
    let right = tf.right();
    let mut velocity = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        velocity += *forward;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        velocity -= *forward;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        velocity -= *right;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        velocity += *right;
    }
    if keyboard.pressed(KeyCode::Space) {
        velocity += Vec3::Y;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) {
        velocity -= Vec3::Y;
    }

    let speed = if keyboard.pressed(KeyCode::ControlLeft) {
        cam.speed * 3.0
    } else {
        cam.speed
    };

    if velocity.length_squared() > 0.0 {
        tf.translation += velocity.normalize() * speed * time.delta_secs();
    }
}
