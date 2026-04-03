use bevy::prelude::IVec3;

/// 6面の方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    PosX = 0,
    NegX = 1,
    PosY = 2,
    NegY = 3,
    PosZ = 4,
    NegZ = 5,
}

impl Face {
    pub const ALL: [Face; 6] = [
        Face::PosX,
        Face::NegX,
        Face::PosY,
        Face::NegY,
        Face::PosZ,
        Face::NegZ,
    ];

    pub fn normal(self) -> IVec3 {
        match self {
            Face::PosX => IVec3::X,
            Face::NegX => IVec3::NEG_X,
            Face::PosY => IVec3::Y,
            Face::NegY => IVec3::NEG_Y,
            Face::PosZ => IVec3::Z,
            Face::NegZ => IVec3::NEG_Z,
        }
    }

    pub fn normal_f32(self) -> [f32; 3] {
        let n = self.normal();
        [n.x as f32, n.y as f32, n.z as f32]
    }
}
