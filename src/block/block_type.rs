use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum BlockType {
    Air = 0,
    Stone = 1,
    Dirt = 2,
    Grass = 3,
    Sand = 4,
    Wood = 5,
    Leaves = 6,
    Cobblestone = 7,
    Planks = 8,
    Glass = 9,
    Bedrock = 10,
}

impl BlockType {
    pub fn is_air(self) -> bool {
        matches!(self, BlockType::Air)
    }

    pub fn is_transparent(self) -> bool {
        matches!(self, BlockType::Air | BlockType::Glass | BlockType::Leaves)
    }

    pub fn is_solid(self) -> bool {
        !matches!(self, BlockType::Air)
    }

    pub fn display_name(self) -> &'static str {
        match self {
            BlockType::Air => "Air",
            BlockType::Stone => "Stone",
            BlockType::Dirt => "Dirt",
            BlockType::Grass => "Grass",
            BlockType::Sand => "Sand",
            BlockType::Wood => "Wood",
            BlockType::Leaves => "Leaves",
            BlockType::Cobblestone => "Cobblestone",
            BlockType::Planks => "Planks",
            BlockType::Glass => "Glass",
            BlockType::Bedrock => "Bedrock",
        }
    }

    /// ブロックの色 (RGB 0.0-1.0) - テクスチャなし時のフォールバック
    pub fn base_color(self) -> [f32; 4] {
        match self {
            BlockType::Air => [0.0, 0.0, 0.0, 0.0],
            BlockType::Stone => [0.5, 0.5, 0.5, 1.0],
            BlockType::Dirt => [0.55, 0.35, 0.15, 1.0],
            BlockType::Grass => [0.3, 0.7, 0.2, 1.0],
            BlockType::Sand => [0.85, 0.8, 0.5, 1.0],
            BlockType::Wood => [0.45, 0.3, 0.1, 1.0],
            BlockType::Leaves => [0.2, 0.6, 0.1, 0.8],
            BlockType::Cobblestone => [0.4, 0.4, 0.4, 1.0],
            BlockType::Planks => [0.7, 0.55, 0.3, 1.0],
            BlockType::Glass => [0.7, 0.9, 1.0, 0.3],
            BlockType::Bedrock => [0.1, 0.1, 0.1, 1.0],
        }
    }
}

impl TryFrom<u8> for BlockType {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(BlockType::Air),
            1 => Ok(BlockType::Stone),
            2 => Ok(BlockType::Dirt),
            3 => Ok(BlockType::Grass),
            4 => Ok(BlockType::Sand),
            5 => Ok(BlockType::Wood),
            6 => Ok(BlockType::Leaves),
            7 => Ok(BlockType::Cobblestone),
            8 => Ok(BlockType::Planks),
            9 => Ok(BlockType::Glass),
            10 => Ok(BlockType::Bedrock),
            _ => Err(()),
        }
    }
}
