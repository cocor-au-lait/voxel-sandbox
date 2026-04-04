use bevy::prelude::*;

pub mod camera;
pub mod collision;
pub mod interaction;
pub mod movement;

#[derive(Component)]
pub struct Player;

/// FPS カメラの回転角度と感度
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

/// プレイヤーの速度ベクトル
#[derive(Component, Default)]
pub struct PlayerVelocity(pub Vec3);

/// 地面に接触しているか
#[derive(Component, Default)]
pub struct PlayerOnGround(pub bool);

/// レイキャスト結果
#[derive(Resource, Default)]
pub struct TargetBlock {
    pub pos: Option<IVec3>,
    pub normal: Option<IVec3>,
}

/// 現在選択中のブロック
#[derive(Resource)]
pub struct SelectedBlock(pub crate::block::BlockType);

impl Default for SelectedBlock {
    fn default() -> Self {
        Self(crate::block::BlockType::Stone)
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TargetBlock>()
            .init_resource::<SelectedBlock>()
            .add_systems(Startup, (camera::spawn_player, interaction::setup_hud))
            .add_systems(
                Update,
                (
                    camera::handle_mouse_look,
                    movement::apply_movement,
                    collision::apply_velocity_with_collision,
                    camera::sync_camera_to_player,
                    interaction::cast_ray,
                    interaction::handle_block_input,
                    interaction::update_highlight,
                    interaction::update_overlay,
                )
                    .chain(),
            );
    }
}
