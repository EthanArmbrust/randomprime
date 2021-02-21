use crate::{pickup_meta::ScriptObjectLocation, custom_asset_ids};
use structs::structs::{
    DamageVulnerability,
    ChargedBeams,
    BeamCombos
};
use reader_writer::{FourCC};
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug)]
pub struct DoorLocation {
    pub door_location: ScriptObjectLocation,
    pub door_force_location: ScriptObjectLocation,
    pub door_shield_location: Option<ScriptObjectLocation>,
    pub dock_number: Option<u32>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum TypeVulnerability {
    Normal = 0x1,
    Reflect = 0x2,
    Immune = 0x3,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum DoorType {
    Blue,
    Purple,
    White,
    Red,
    PowerBomb,
    Bomb,
    Boost,
    Missile,
    Charge,
    Super,
    Wavebuster,
    Icespreader,
    Flamethrower,
    Ai,
    Disabled,
    VerticalBlue,
    VerticalPurple,
    VerticalWhite,
    VerticalRed,
    VerticalPowerBomb,
    VerticalBomb,
    VerticalMissile,
    VerticalCharge,
    VerticalSuper,
    VerticalDisabled,
    VerticalWavebuster,
    VerticalIcespreader,
    VerticalFlamethrower,
    VerticalAi,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Weights {
    pub tallon_overworld: [u8;4],
    pub chozo_ruins: [u8;4],
    pub magmoor_caverns: [u8;4],
    pub phendrana_drifts: [u8;4],
    pub phazon_mines: [u8;4]
}

pub enum World {
    FrigateOrpheon,
    TallonOverworld,
    ChozoRuins,
    MagmoorCaverns,
    PhendranaDrifts,
    PhazonMines,
    ImpactCrater,
}

impl World {
    pub fn from_pak(pak_str:&str) -> Option<Self> {
        match pak_str {
            "Metroid1.pak" => Some(World::FrigateOrpheon),
            "Metroid2.pak" => Some(World::ChozoRuins),
            "Metroid3.pak" => Some(World::PhendranaDrifts),
            "Metroid4.pak" => Some(World::TallonOverworld),
            "metroid5.pak" => Some(World::PhazonMines),
            "Metroid6.pak" => Some(World::MagmoorCaverns),
            "Metroid7.pak" => Some(World::ImpactCrater),
            _ => None
        }
    }
}

impl DoorType {

    pub const fn is_vertical(&self) -> bool {
        match self {
            DoorType::VerticalBlue         =>   true,
            DoorType::VerticalPurple       =>   true,
            DoorType::VerticalWhite        =>   true,
            DoorType::VerticalRed          =>   true,
            DoorType::VerticalPowerBomb    =>   true,
            DoorType::VerticalBomb         =>   true,
            DoorType::VerticalMissile      =>   true,
            DoorType::VerticalCharge       =>   true,
            DoorType::VerticalSuper        =>   true,
            DoorType::VerticalDisabled     =>   true,
            DoorType::VerticalWavebuster   =>   true,
            DoorType::VerticalIcespreader  =>   true,
            DoorType::VerticalFlamethrower =>   true,
            DoorType::VerticalAi           =>   true,
            _ => false,
        }
    }

    pub const fn shield_cmdl(&self) -> u32 { // model of door, includes specification for which 128x128 texture to line door frame with
        match self {
            DoorType::Blue         =>   0x0734977A, // vanilla CMDL - "blueShield_v1" - door frame model
            DoorType::Purple       =>   0x33188D1B, // vanilla CMDL
            DoorType::White        =>   0x59649E9D, // vanilla CMDL
            DoorType::Red          =>   0xBBBA1EC7, // vanilla CMDL
            DoorType::Boost        =>   0x0734977A, // unused
            DoorType::PowerBomb    =>   custom_asset_ids::POWER_BOMB_DOOR_CMDL,
            DoorType::Bomb         =>   custom_asset_ids::MORPH_BALL_BOMB_DOOR_CMDL,
            DoorType::Missile      =>   custom_asset_ids::MISSILE_DOOR_CMDL,
            DoorType::Charge       =>   custom_asset_ids::CHARGE_DOOR_CMDL,
            DoorType::Super        =>   custom_asset_ids::SUPER_MISSILE_DOOR_CMDL,
            DoorType::Disabled     =>   custom_asset_ids::DISABLED_DOOR_CMDL,
            DoorType::Wavebuster   =>   custom_asset_ids::WAVEBUSTER_DOOR_CMDL,
            DoorType::Icespreader  =>   custom_asset_ids::ICESPREADER_DOOR_CMDL,
            DoorType::Flamethrower =>   custom_asset_ids::FLAMETHROWER_DOOR_CMDL,
            DoorType::Ai           =>   custom_asset_ids::AI_DOOR_CMDL,

            // vertical doors need a different CMDL, otherwise it will look like this: https://i.imgur.com/jGjWnmg.png //
            DoorType::VerticalBlue         =>   0x18D0AEE6, // vanilla horizontal CMDL (blue)
            DoorType::VerticalPurple       =>   0x095B0B93, // vanilla CMDL
            DoorType::VerticalWhite        =>   0xB7A8A4C9, // vanilla CMDL
            DoorType::VerticalRed          =>   custom_asset_ids::VERTICAL_RED_DOOR_CMDL, // vanilla CMDL
            DoorType::VerticalPowerBomb    =>   custom_asset_ids::VERTICAL_POWER_BOMB_DOOR_CMDL,
            DoorType::VerticalBomb         =>   custom_asset_ids::VERTICAL_MORPH_BALL_BOMB_DOOR_CMDL,
            DoorType::VerticalMissile      =>   custom_asset_ids::VERTICAL_MISSILE_DOOR_CMDL,
            DoorType::VerticalCharge       =>   custom_asset_ids::VERTICAL_CHARGE_DOOR_CMDL,
            DoorType::VerticalSuper        =>   custom_asset_ids::VERTICAL_SUPER_MISSILE_DOOR_CMDL,
            DoorType::VerticalDisabled     =>   custom_asset_ids::VERTICAL_DISABLED_DOOR_CMDL,
            DoorType::VerticalWavebuster   =>   custom_asset_ids::VERTICAL_WAVEBUSTER_DOOR_CMDL,
            DoorType::VerticalIcespreader  =>   custom_asset_ids::VERTICAL_ICESPREADER_DOOR_CMDL,
            DoorType::VerticalFlamethrower =>   custom_asset_ids::VERTICAL_FLAMETHROWER_DOOR_CMDL,
            DoorType::VerticalAi           =>   custom_asset_ids::VERTICAL_AI_DOOR_CMDL,
        }
    }

    pub const fn map_object_type(&self) -> u32 {
        match self {
            DoorType::Blue          => structs::MapaObjectType::DoorNormal as u32,
            DoorType::Charge        => structs::MapaObjectType::DoorNormal as u32,
            DoorType::Bomb          => structs::MapaObjectType::DoorNormal as u32,
            DoorType::Purple        => structs::MapaObjectType::DoorWave as u32,
            DoorType::Wavebuster    => structs::MapaObjectType::DoorWave as u32,
            DoorType::White         => structs::MapaObjectType::DoorIce as u32,
            DoorType::Icespreader   => structs::MapaObjectType::DoorIce as u32,
            DoorType::Red           => structs::MapaObjectType::DoorPlasma as u32,
            DoorType::Flamethrower  => structs::MapaObjectType::DoorPlasma as u32,
            _ => structs::MapaObjectType::DoorShield as u32, // everything else is non-vanilla and thus shield
        }
    }

    pub const fn forcefield_txtr(&self) -> u32 { // texture to scroll across center of door for "forcefield" effect 16x16
        match self {
            DoorType::Blue         =>   0x8A7F3683, // vanilla TXTR - blue 16x16
            DoorType::Purple       =>   0xF68DF7F1, // vanilla TXTR
            DoorType::White        =>   0xBE4CD99D, // vanilla TXTR
            DoorType::Red          =>   0xFC095F6C, // vanilla TXTR
            DoorType::Boost        =>   0x8A7F3683, // unused
            DoorType::PowerBomb    =>   0x1D588B22, // solid yellow
            DoorType::Bomb         =>   0xFC095F6C, // solid orange
            DoorType::Missile      =>   0x8344BEC8, // solid grey
            DoorType::Charge       =>   0x8A7F3683, // vanilla blue
            DoorType::Super        =>   0xD5C17775, // solid green
            DoorType::Disabled     =>   0x717AABCE, // void with specks
            DoorType::Wavebuster   =>   0xF68DF7F1, // vanilla TXTR
            DoorType::Icespreader  =>   0xBE4CD99D, // vanilla TXTR
            DoorType::Flamethrower =>   0xFC095F6C, // vanilla TXTR
            DoorType::Ai           =>   0x717AABCE, // void with specks

            // vertical doors use the same textures as their horizontal variants //
            DoorType::VerticalBlue         =>   DoorType::Blue.forcefield_txtr(),
            DoorType::VerticalPurple       =>   DoorType::Purple.forcefield_txtr(),
            DoorType::VerticalWhite        =>   DoorType::White.forcefield_txtr(),
            DoorType::VerticalRed          =>   DoorType::Red.forcefield_txtr(),
            DoorType::VerticalPowerBomb    =>   DoorType::PowerBomb.forcefield_txtr(),
            DoorType::VerticalBomb         =>   DoorType::Bomb.forcefield_txtr(),         
            DoorType::VerticalMissile      =>   DoorType::Missile.forcefield_txtr(), 
            DoorType::VerticalCharge       =>   DoorType::Charge.forcefield_txtr(), 
            DoorType::VerticalSuper        =>   DoorType::Super.forcefield_txtr(), 
            DoorType::VerticalDisabled     =>   DoorType::Disabled.forcefield_txtr(), 
            DoorType::VerticalWavebuster   =>   DoorType::Wavebuster.forcefield_txtr(), 
            DoorType::VerticalIcespreader  =>   DoorType::Icespreader.forcefield_txtr(), 
            DoorType::VerticalFlamethrower =>   DoorType::Flamethrower.forcefield_txtr(), 
            DoorType::VerticalAi           =>   DoorType::Ai.forcefield_txtr(), 
        }
    }

    pub fn holorim_texture(&self) -> u32 { // The the color applied from the rim of the door frame, specified in CMDL
        match self {
            DoorType::Blue         =>   0x88ED4593, // vanilla TXTR - "blueholorim" texture [128x128]
            DoorType::Purple       =>   0xAB031EA9, // vanilla TXTR
            DoorType::White        =>   0xF6870C9F, // vanilla TXTR
            DoorType::Red          =>   0x61A6945B, // vanilla TXTR
            DoorType::Boost        =>   0x88ED4593, // unused
            DoorType::PowerBomb    =>   custom_asset_ids::POWER_BOMB_DOOR_TXTR,
            DoorType::Bomb         =>   custom_asset_ids::MORPH_BALL_BOMB_DOOR_TXTR,
            DoorType::Missile      =>   0x459582C1, // "bedroomeyesC"
            DoorType::Charge       =>   0xC7C8AF66, // banded blue ribbon
            DoorType::Super        =>   custom_asset_ids::SUPER_MISSILE_DOOR_TXTR,
            DoorType::Wavebuster   =>   custom_asset_ids::WAVEBUSTER_DOOR_TXTR,
            DoorType::Icespreader  =>   custom_asset_ids::ICESPREADER_DOOR_TXTR,
            DoorType::Flamethrower =>   custom_asset_ids::FLAMETHROWER_DOOR_TXTR,
            DoorType::Disabled     =>   0x717AABCE, // void with specks
            DoorType::Ai           =>   custom_asset_ids::AI_DOOR_TXTR,
            
            // vertical doors use the same textures as their horizontal variants //
            DoorType::VerticalBlue         =>   DoorType::Blue.holorim_texture(),
            DoorType::VerticalPurple       =>   DoorType::Purple.holorim_texture(),
            DoorType::VerticalWhite        =>   DoorType::White.holorim_texture(),
            DoorType::VerticalRed          =>   DoorType::Red.holorim_texture(),
            DoorType::VerticalPowerBomb    =>   DoorType::PowerBomb.holorim_texture(),
            DoorType::VerticalBomb         =>   DoorType::Bomb.holorim_texture(),         
            DoorType::VerticalMissile      =>   DoorType::Missile.holorim_texture(), 
            DoorType::VerticalCharge       =>   DoorType::Charge.holorim_texture(), 
            DoorType::VerticalSuper        =>   DoorType::Super.holorim_texture(), 
            DoorType::VerticalDisabled     =>   DoorType::Disabled.holorim_texture(), 
            DoorType::VerticalWavebuster   =>   DoorType::Wavebuster.holorim_texture(), 
            DoorType::VerticalIcespreader  =>   DoorType::Icespreader.holorim_texture(), 
            DoorType::VerticalFlamethrower =>   DoorType::Flamethrower.holorim_texture(), 
            DoorType::VerticalAi           =>   DoorType::Ai.holorim_texture(),
        }
    }

    pub fn dependencies(&self) -> Vec<(u32, FourCC)> { // dependencies to add to the area
        
        let mut data: Vec<(u32, FourCC)> = Vec::new();
        data.push((self.shield_cmdl(),FourCC::from_bytes(b"CMDL")));
        data.push((self.forcefield_txtr(),FourCC::from_bytes(b"TXTR")));
        if self.holorim_texture() != 0x00000000 {
            data.push((self.holorim_texture(),FourCC::from_bytes(b"TXTR")));
        }

        // If the door is a t-posing chozo ghost, add that models dependencies as well
        if self.shield_cmdl() == 0xDAAC77CB {
            data.push((0xB516D300,FourCC::from_bytes(b"TXTR")));
            data.push((0x8D4EF1D8,FourCC::from_bytes(b"TXTR")));
            data.push((0x7D81B904,FourCC::from_bytes(b"TXTR")));
        }

        data
    }

    pub fn iter() -> impl Iterator<Item = DoorType> {
        [
            DoorType::Blue,
            DoorType::Purple,
            DoorType::White,
            DoorType::Red,
            DoorType::PowerBomb,
            DoorType::Bomb,
            DoorType::Boost,
            DoorType::Missile,
            DoorType::Charge,
            DoorType::Super,
            DoorType::Disabled,
            DoorType::Wavebuster,
            DoorType::Icespreader,
            DoorType::Flamethrower,
            DoorType::Ai,
            DoorType::VerticalBlue,
            DoorType::VerticalPurple,
            DoorType::VerticalWhite,
            DoorType::VerticalRed,
            DoorType::VerticalPowerBomb,
            DoorType::VerticalBomb,
            DoorType::VerticalMissile,
            DoorType::VerticalCharge,
            DoorType::VerticalSuper,
            DoorType::VerticalDisabled,
            DoorType::VerticalWavebuster,
            DoorType::VerticalIcespreader,
            DoorType::VerticalFlamethrower,
            DoorType::VerticalAi,
        ].iter().map(|i| *i)
    }

    pub fn vulnerability(&self) -> DamageVulnerability {
        match self {
            DoorType::Blue => DamageVulnerability {
                power: TypeVulnerability::Normal as u32,
                ice: TypeVulnerability::Normal as u32,
                wave: TypeVulnerability::Normal as u32,
                plasma: TypeVulnerability::Normal as u32,
                bomb: TypeVulnerability::Normal as u32,
                power_bomb: TypeVulnerability::Normal as u32,
                missile: TypeVulnerability::Normal as u32,
                boost_ball: TypeVulnerability::Reflect as u32,
                phazon: TypeVulnerability::Normal as u32,

                enemy_weapon0:TypeVulnerability::Immune as u32,
                enemy_weapon1:TypeVulnerability::Immune as u32,
                enemy_weapon2:TypeVulnerability::Immune as u32,
                enemy_weapon3:TypeVulnerability::Immune as u32,

                unknown_weapon0:TypeVulnerability::Immune as u32,
                unknown_weapon1:TypeVulnerability::Immune as u32,
                unknown_weapon2:TypeVulnerability::Immune as u32,

                charged_beams:ChargedBeams {
                    power:TypeVulnerability::Normal as u32,
                    ice:TypeVulnerability::Normal as u32,
                    wave:TypeVulnerability::Normal as u32,
                    plasma:TypeVulnerability::Normal as u32,
                    phazon:TypeVulnerability::Normal as u32,
                },
                beam_combos:BeamCombos {
                    power:TypeVulnerability::Normal as u32,
                    ice:TypeVulnerability::Normal as u32,
                    wave:TypeVulnerability::Normal as u32,
                    plasma:TypeVulnerability::Normal as u32,
                    phazon:TypeVulnerability::Normal as u32,
                }
            },
            DoorType::Purple => DamageVulnerability {
                power: TypeVulnerability::Reflect as u32,
                ice: TypeVulnerability::Reflect as u32,
                wave: TypeVulnerability::Normal as u32,
                plasma: TypeVulnerability::Reflect as u32,
                bomb: TypeVulnerability::Immune as u32,
                power_bomb: TypeVulnerability::Immune as u32,
                missile: TypeVulnerability::Reflect as u32,
                boost_ball: TypeVulnerability::Reflect as u32,
                phazon: TypeVulnerability::Immune as u32,

                enemy_weapon0:TypeVulnerability::Immune as u32,
                enemy_weapon1:TypeVulnerability::Immune as u32,
                enemy_weapon2:TypeVulnerability::Immune as u32,
                enemy_weapon3:TypeVulnerability::Immune as u32,

                unknown_weapon0:TypeVulnerability::Immune as u32,
                unknown_weapon1:TypeVulnerability::Immune as u32,
                unknown_weapon2:TypeVulnerability::Immune as u32,

                charged_beams:ChargedBeams {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Normal as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
                beam_combos:BeamCombos {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Normal as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                }
            },
            DoorType::White => DamageVulnerability {
                power: TypeVulnerability::Reflect as u32,
                ice: TypeVulnerability::Normal as u32,
                wave: TypeVulnerability::Reflect as u32,
                plasma: TypeVulnerability::Reflect as u32,
                bomb: TypeVulnerability::Immune as u32,
                power_bomb: TypeVulnerability::Immune as u32,
                missile: TypeVulnerability::Reflect as u32,
                boost_ball: TypeVulnerability::Reflect as u32,
                phazon: TypeVulnerability::Immune as u32,

                enemy_weapon0:TypeVulnerability::Immune as u32,
                enemy_weapon1:TypeVulnerability::Immune as u32,
                enemy_weapon2:TypeVulnerability::Immune as u32,
                enemy_weapon3:TypeVulnerability::Immune as u32,

                unknown_weapon0:TypeVulnerability::Immune as u32,
                unknown_weapon1:TypeVulnerability::Immune as u32,
                unknown_weapon2:TypeVulnerability::Immune as u32,


                charged_beams:ChargedBeams {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Normal as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
                beam_combos:BeamCombos {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Normal as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                }
            },
            DoorType::Red => DamageVulnerability {
                power: TypeVulnerability::Reflect as u32,
                ice: TypeVulnerability::Reflect as u32,
                wave: TypeVulnerability::Reflect as u32,
                plasma: TypeVulnerability::Normal as u32,
                bomb: TypeVulnerability::Immune as u32,
                power_bomb: TypeVulnerability::Immune as u32,
                missile: TypeVulnerability::Reflect as u32,
                boost_ball: TypeVulnerability::Reflect as u32,
                phazon: TypeVulnerability::Immune as u32,

                enemy_weapon0:TypeVulnerability::Immune as u32,
                enemy_weapon1:TypeVulnerability::Immune as u32,
                enemy_weapon2:TypeVulnerability::Immune as u32,
                enemy_weapon3:TypeVulnerability::Immune as u32,


                unknown_weapon0:TypeVulnerability::Immune as u32,
                unknown_weapon1:TypeVulnerability::Immune as u32,
                unknown_weapon2:TypeVulnerability::Immune as u32,


                charged_beams:ChargedBeams {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Normal as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
                beam_combos:BeamCombos {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Normal as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
            },
            DoorType::PowerBomb => DamageVulnerability {
                power: TypeVulnerability::Reflect as u32,
                ice: TypeVulnerability::Reflect as u32,
                wave: TypeVulnerability::Reflect as u32,
                plasma: TypeVulnerability::Reflect as u32,
                bomb: TypeVulnerability::Immune as u32,
                power_bomb: TypeVulnerability::Normal as u32,
                missile: TypeVulnerability::Reflect as u32,
                boost_ball: TypeVulnerability::Reflect as u32,
                phazon: TypeVulnerability::Immune as u32,

                enemy_weapon0:TypeVulnerability::Immune as u32,
                enemy_weapon1:TypeVulnerability::Immune as u32,
                enemy_weapon2:TypeVulnerability::Immune as u32,
                enemy_weapon3:TypeVulnerability::Immune as u32,

                unknown_weapon0:TypeVulnerability::Immune as u32,
                unknown_weapon1:TypeVulnerability::Immune as u32,
                unknown_weapon2:TypeVulnerability::Immune as u32,

                charged_beams:ChargedBeams {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
                beam_combos:BeamCombos {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
            },
            DoorType::Bomb => DamageVulnerability {
                power: TypeVulnerability::Reflect as u32,
                ice: TypeVulnerability::Reflect as u32,
                wave: TypeVulnerability::Reflect as u32,
                plasma: TypeVulnerability::Reflect as u32,
                bomb: TypeVulnerability::Normal as u32,
                power_bomb: TypeVulnerability::Immune as u32,
                missile: TypeVulnerability::Reflect as u32,
                boost_ball: TypeVulnerability::Reflect as u32,
                phazon: TypeVulnerability::Immune as u32,

                enemy_weapon0:TypeVulnerability::Immune as u32,
                enemy_weapon1:TypeVulnerability::Immune as u32,
                enemy_weapon2:TypeVulnerability::Immune as u32,
                enemy_weapon3:TypeVulnerability::Immune as u32,

                unknown_weapon0:TypeVulnerability::Immune as u32,
                unknown_weapon1:TypeVulnerability::Immune as u32,
                unknown_weapon2:TypeVulnerability::Immune as u32,

                charged_beams:ChargedBeams {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
                beam_combos:BeamCombos {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
            },
            DoorType::Boost => DamageVulnerability {
                power: TypeVulnerability::Reflect as u32,
                ice: TypeVulnerability::Reflect as u32,
                wave: TypeVulnerability::Reflect as u32,
                plasma: TypeVulnerability::Reflect as u32,
                bomb: TypeVulnerability::Immune as u32,
                power_bomb: TypeVulnerability::Immune as u32,
                missile: TypeVulnerability::Reflect as u32,
                boost_ball: TypeVulnerability::Normal as u32,
                phazon: TypeVulnerability::Immune as u32,

                enemy_weapon0:TypeVulnerability::Immune as u32,
                enemy_weapon1:TypeVulnerability::Immune as u32,
                enemy_weapon2:TypeVulnerability::Immune as u32,
                enemy_weapon3:TypeVulnerability::Immune as u32,

                unknown_weapon0:TypeVulnerability::Immune as u32,
                unknown_weapon1:TypeVulnerability::Immune as u32,
                unknown_weapon2:TypeVulnerability::Immune as u32,

                charged_beams:ChargedBeams {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
                beam_combos:BeamCombos {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
            },
            DoorType::Missile => DamageVulnerability {
                power: TypeVulnerability::Reflect as u32,
                ice: TypeVulnerability::Reflect as u32,
                wave: TypeVulnerability::Reflect as u32,
                plasma: TypeVulnerability::Reflect as u32,
                bomb: TypeVulnerability::Immune as u32,
                power_bomb: TypeVulnerability::Immune as u32,
                missile: TypeVulnerability::Normal as u32,
                boost_ball: TypeVulnerability::Reflect as u32,
                phazon: TypeVulnerability::Immune as u32,

                enemy_weapon0:TypeVulnerability::Immune as u32,
                enemy_weapon1:TypeVulnerability::Immune as u32,
                enemy_weapon2:TypeVulnerability::Immune as u32,
                enemy_weapon3:TypeVulnerability::Immune as u32,

                unknown_weapon0:TypeVulnerability::Immune as u32,
                unknown_weapon1:TypeVulnerability::Immune as u32,
                unknown_weapon2:TypeVulnerability::Immune as u32,

                charged_beams:ChargedBeams {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
                beam_combos:BeamCombos {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
            },
            DoorType::Charge => DamageVulnerability {
                power: TypeVulnerability::Reflect as u32,
                ice: TypeVulnerability::Reflect as u32,
                wave: TypeVulnerability::Reflect as u32,
                plasma: TypeVulnerability::Reflect as u32,
                bomb: TypeVulnerability::Immune as u32,
                power_bomb: TypeVulnerability::Immune as u32,
                missile: TypeVulnerability::Reflect as u32,
                boost_ball: TypeVulnerability::Reflect as u32,
                phazon: TypeVulnerability::Immune as u32,

                enemy_weapon0:TypeVulnerability::Immune as u32,
                enemy_weapon1:TypeVulnerability::Immune as u32,
                enemy_weapon2:TypeVulnerability::Immune as u32,
                enemy_weapon3:TypeVulnerability::Immune as u32,

                unknown_weapon0:TypeVulnerability::Immune as u32,
                unknown_weapon1:TypeVulnerability::Immune as u32,
                unknown_weapon2:TypeVulnerability::Immune as u32,

                charged_beams:ChargedBeams {
                    power:TypeVulnerability::Normal as u32,
                    ice:TypeVulnerability::Normal as u32,
                    wave:TypeVulnerability::Normal as u32,
                    plasma:TypeVulnerability::Normal as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
                beam_combos:BeamCombos {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
            },
            DoorType::Super => DamageVulnerability {
                power: TypeVulnerability::Reflect as u32,
                ice: TypeVulnerability::Reflect as u32,
                wave: TypeVulnerability::Reflect as u32,
                plasma: TypeVulnerability::Reflect as u32,
                bomb: TypeVulnerability::Immune as u32,
                power_bomb: TypeVulnerability::Immune as u32,
                missile: TypeVulnerability::Reflect as u32,
                boost_ball: TypeVulnerability::Reflect as u32,
                phazon: TypeVulnerability::Immune as u32,

                enemy_weapon0:TypeVulnerability::Immune as u32,
                enemy_weapon1:TypeVulnerability::Immune as u32,
                enemy_weapon2:TypeVulnerability::Immune as u32,
                enemy_weapon3:TypeVulnerability::Immune as u32,

                unknown_weapon0:TypeVulnerability::Immune as u32,
                unknown_weapon1:TypeVulnerability::Immune as u32,
                unknown_weapon2:TypeVulnerability::Immune as u32,

                charged_beams:ChargedBeams {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
                beam_combos:BeamCombos {
                    power:TypeVulnerability::Normal as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
            },
            DoorType::Wavebuster => DamageVulnerability {
                power: TypeVulnerability::Reflect as u32,
                ice: TypeVulnerability::Reflect as u32,
                wave: TypeVulnerability::Reflect as u32,
                plasma: TypeVulnerability::Reflect as u32,
                bomb: TypeVulnerability::Immune as u32,
                power_bomb: TypeVulnerability::Immune as u32,
                missile: TypeVulnerability::Reflect as u32,
                boost_ball: TypeVulnerability::Reflect as u32,
                phazon: TypeVulnerability::Immune as u32,

                enemy_weapon0:TypeVulnerability::Immune as u32,
                enemy_weapon1:TypeVulnerability::Immune as u32,
                enemy_weapon2:TypeVulnerability::Immune as u32,
                enemy_weapon3:TypeVulnerability::Immune as u32,

                unknown_weapon0:TypeVulnerability::Immune as u32,
                unknown_weapon1:TypeVulnerability::Immune as u32,
                unknown_weapon2:TypeVulnerability::Immune as u32,

                charged_beams:ChargedBeams {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
                beam_combos:BeamCombos {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Normal as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
            },
            DoorType::Icespreader => DamageVulnerability {
                power: TypeVulnerability::Reflect as u32,
                ice: TypeVulnerability::Reflect as u32,
                wave: TypeVulnerability::Reflect as u32,
                plasma: TypeVulnerability::Reflect as u32,
                bomb: TypeVulnerability::Immune as u32,
                power_bomb: TypeVulnerability::Immune as u32,
                missile: TypeVulnerability::Reflect as u32,
                boost_ball: TypeVulnerability::Reflect as u32,
                phazon: TypeVulnerability::Immune as u32,

                enemy_weapon0:TypeVulnerability::Immune as u32,
                enemy_weapon1:TypeVulnerability::Immune as u32,
                enemy_weapon2:TypeVulnerability::Immune as u32,
                enemy_weapon3:TypeVulnerability::Immune as u32,

                unknown_weapon0:TypeVulnerability::Immune as u32,
                unknown_weapon1:TypeVulnerability::Immune as u32,
                unknown_weapon2:TypeVulnerability::Immune as u32,

                charged_beams:ChargedBeams {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
                beam_combos:BeamCombos {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Normal as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
            },
            DoorType::Flamethrower => DamageVulnerability {
                power: TypeVulnerability::Reflect as u32,
                ice: TypeVulnerability::Reflect as u32,
                wave: TypeVulnerability::Reflect as u32,
                plasma: TypeVulnerability::Reflect as u32,
                bomb: TypeVulnerability::Immune as u32,
                power_bomb: TypeVulnerability::Immune as u32,
                missile: TypeVulnerability::Reflect as u32,
                boost_ball: TypeVulnerability::Reflect as u32,
                phazon: TypeVulnerability::Immune as u32,

                enemy_weapon0:TypeVulnerability::Immune as u32,
                enemy_weapon1:TypeVulnerability::Immune as u32,
                enemy_weapon2:TypeVulnerability::Immune as u32,
                enemy_weapon3:TypeVulnerability::Immune as u32,

                unknown_weapon0:TypeVulnerability::Immune as u32,
                unknown_weapon1:TypeVulnerability::Immune as u32,
                unknown_weapon2:TypeVulnerability::Immune as u32,

                charged_beams:ChargedBeams {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
                beam_combos:BeamCombos {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Normal as u32,
                    phazon:TypeVulnerability::Reflect as u32,
                },
            },
            DoorType::Disabled => DamageVulnerability {
                power: TypeVulnerability::Immune as u32,
                ice: TypeVulnerability::Immune as u32,
                wave: TypeVulnerability::Immune as u32,
                plasma: TypeVulnerability::Immune as u32,
                bomb: TypeVulnerability::Immune as u32,
                power_bomb: TypeVulnerability::Immune as u32,
                missile: TypeVulnerability::Immune as u32,
                boost_ball: TypeVulnerability::Immune as u32,
                phazon: TypeVulnerability::Normal as u32,
                
                enemy_weapon0:TypeVulnerability::Immune as u32,
                enemy_weapon1:TypeVulnerability::Immune as u32,
                enemy_weapon2:TypeVulnerability::Immune as u32,
                enemy_weapon3:TypeVulnerability::Immune as u32,

                unknown_weapon0:TypeVulnerability::Immune as u32,
                unknown_weapon1:TypeVulnerability::Immune as u32,
                unknown_weapon2:TypeVulnerability::Immune as u32,

                charged_beams:ChargedBeams {
                    power:TypeVulnerability::Immune as u32,
                    ice:TypeVulnerability::Immune as u32,
                    wave:TypeVulnerability::Immune as u32,
                    plasma:TypeVulnerability::Immune as u32,
                    phazon:TypeVulnerability::Normal as u32,
                },
                beam_combos:BeamCombos {
                    power:TypeVulnerability::Immune as u32,
                    ice:TypeVulnerability::Immune as u32,
                    wave:TypeVulnerability::Immune as u32,
                    plasma:TypeVulnerability::Immune as u32,
                    phazon:TypeVulnerability::Normal as u32,
                },
            },
            DoorType::Ai => DamageVulnerability {
                power: TypeVulnerability::Reflect as u32,
                ice: TypeVulnerability::Reflect as u32,
                wave: TypeVulnerability::Reflect as u32,
                plasma: TypeVulnerability::Reflect as u32,
                bomb: TypeVulnerability::Immune as u32,
                power_bomb: TypeVulnerability::Immune as u32,
                missile: TypeVulnerability::Reflect as u32,
                boost_ball: TypeVulnerability::Immune as u32,
                phazon: TypeVulnerability::Normal as u32,
                
                enemy_weapon0:TypeVulnerability::Normal as u32,
                enemy_weapon1:TypeVulnerability::Normal as u32,
                enemy_weapon2:TypeVulnerability::Normal as u32,
                enemy_weapon3:TypeVulnerability::Normal as u32,

                unknown_weapon0:TypeVulnerability::Normal as u32,
                unknown_weapon1:TypeVulnerability::Normal as u32,
                unknown_weapon2:TypeVulnerability::Normal as u32,

                charged_beams:ChargedBeams {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Normal as u32,
                },
                beam_combos:BeamCombos {
                    power:TypeVulnerability::Reflect as u32,
                    ice:TypeVulnerability::Reflect as u32,
                    wave:TypeVulnerability::Reflect as u32,
                    plasma:TypeVulnerability::Reflect as u32,
                    phazon:TypeVulnerability::Normal as u32,
                },
            },

            // vertical doors use the same damage vulnerabilites as their horizontal variants //
            DoorType::VerticalBlue         =>   DoorType::Blue.vulnerability(),
            DoorType::VerticalPurple       =>   DoorType::Purple.vulnerability(),
            DoorType::VerticalWhite        =>   DoorType::White.vulnerability(),
            DoorType::VerticalRed          =>   DoorType::Red.vulnerability(),
            DoorType::VerticalPowerBomb    =>   DoorType::PowerBomb.vulnerability(),
            DoorType::VerticalBomb         =>   DoorType::Bomb.vulnerability(),         
            DoorType::VerticalMissile      =>   DoorType::Missile.vulnerability(), 
            DoorType::VerticalCharge       =>   DoorType::Charge.vulnerability(), 
            DoorType::VerticalSuper        =>   DoorType::Super.vulnerability(), 
            DoorType::VerticalDisabled     =>   DoorType::Disabled.vulnerability(), 
            DoorType::VerticalWavebuster   =>   DoorType::Wavebuster.vulnerability(), 
            DoorType::VerticalIcespreader  =>   DoorType::Icespreader.vulnerability(), 
            DoorType::VerticalFlamethrower =>   DoorType::Flamethrower.vulnerability(), 
            DoorType::VerticalAi           =>   DoorType::Ai.vulnerability(),
        }
    }

    pub fn from_cmdl (cmdl: &u32) -> Option<Self> {
        match cmdl {
            0x0734977A => Some(DoorType::Blue),
            0x33188D1B => Some(DoorType::Purple),
            0x59649E9D => Some(DoorType::White),
            0xBBBA1EC7 => Some(DoorType::Red),
            _ => None,
        }
    }

    pub fn from_txtr (txtr: &u32) -> Option<Self> {
        match txtr {
            0x8A7F3683 => Some(DoorType::Blue),
            0xF68DF7F1 => Some(DoorType::Purple),
            0xBE4CD99D => Some(DoorType::White),
            0xFC095F6C => Some(DoorType::Red),
            _ => None,
        }
    }
}
