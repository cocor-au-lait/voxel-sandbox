use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use super::{Player, PlayerOnGround, PlayerVelocity};

#[derive(Component)]
pub struct PlayerCamera {
    pub pitch: f32,
    pub yaw: f32,
    pub sensitivity: f32,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            pitch: 0.0,
            yaw: 0.0,
            sensitivity: 0.002,
        }
    }
}

pub fn spawn_player(mut commands: Commands) {
    // プレイヤー本体 (足元位置)
    commands
        .spawn((
            Player,
            PlayerVelocity::default(),
            PlayerOnGround::default(),
            Transform::from_xyz(16.0, 68.0, 16.0),
        ))
        .with_children(|parent| {
            // カメラ (目の高さ: 1.65m)
            parent.spawn((
                PlayerCamera::default(),
                Camera3d::default(),
                Transform::from_xyz(0.0, 1.65, 0.0),
            ));
        });
}

pub fn handle_mouse_look(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut camera_q: Query<(&mut Transform, &mut PlayerCamera)>,
) {
    let Ok(mut cursor) = cursor_q.single_mut() else {
        return;
    };

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
}
