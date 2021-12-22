use crate::{
    custom_assets::custom_asset_ids,
    structs::scly_props::structs::{DamageVulnerability, BeamCombos, ChargedBeams},
};

use structs::{res_id, ResId};
use reader_writer::{FourCC};

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
    PowerOnly,
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
    VerticalPowerOnly,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum BlastShieldType {
    Missile,
    PowerBomb,
    Super,
    Wavebuster,
    Icespreader,
    Flamethrower,
}

impl DoorType {

    pub const fn is_vertical(&self) -> bool {
        match self {
            DoorType::VerticalBlue         =>   true,
            DoorType::VerticalPowerOnly    =>   true,
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

    pub fn to_vertical(&self) -> DoorType {
        match self {
            DoorType::Blue         =>   DoorType::VerticalBlue         ,
            DoorType::PowerOnly    =>   DoorType::VerticalPowerOnly    ,
            DoorType::Purple       =>   DoorType::VerticalPurple       ,
            DoorType::White        =>   DoorType::VerticalWhite        ,
            DoorType::Red          =>   DoorType::VerticalRed          ,
            DoorType::PowerBomb    =>   DoorType::VerticalPowerBomb    ,
            DoorType::Bomb         =>   DoorType::VerticalBomb         ,
            DoorType::Missile      =>   DoorType::VerticalMissile      ,
            DoorType::Charge       =>   DoorType::VerticalCharge       ,
            DoorType::Super        =>   DoorType::VerticalSuper        ,
            DoorType::Disabled     =>   DoorType::VerticalDisabled     ,
            DoorType::Wavebuster   =>   DoorType::VerticalWavebuster   ,
            DoorType::Icespreader  =>   DoorType::VerticalIcespreader  ,
            DoorType::Flamethrower =>   DoorType::VerticalFlamethrower ,
            DoorType::Ai           =>   DoorType::VerticalAi           ,
            _ => self.clone().to_owned(),
        }
    }

    pub fn from_string(string: String) -> Option<Self> {
        match string.trim().to_lowercase().replace(" ","").replace("_", "").as_str() {
            "blue"          => Some(DoorType::Blue         ),
            "poweronly"     => Some(DoorType::PowerOnly    ),
            "purple"        => Some(DoorType::Purple       ),
            "wave"          => Some(DoorType::Purple       ),
            "wavebeam"      => Some(DoorType::Purple       ),
            "white"         => Some(DoorType::White        ),
            "ice"           => Some(DoorType::White        ),
            "icebeam"       => Some(DoorType::White        ),
            "red"           => Some(DoorType::Red          ),
            "plasma"        => Some(DoorType::Red          ),
            "plasmabeam"    => Some(DoorType::Red          ),
            "powerbomb"     => Some(DoorType::PowerBomb    ),
            "bomb"          => Some(DoorType::Bomb         ),
            "bombs"         => Some(DoorType::Bomb         ),
            "missile"       => Some(DoorType::Missile      ),
            "missiles"      => Some(DoorType::Missile      ),
            "charge"        => Some(DoorType::Charge       ),
            "chargebeam"    => Some(DoorType::Charge       ),
            "super"         => Some(DoorType::Super        ),
            "supermissile"  => Some(DoorType::Super        ),
            "supermissiles" => Some(DoorType::Super        ),
            "disabled"      => Some(DoorType::Disabled     ),
            "wavebuster"    => Some(DoorType::Wavebuster   ),
            "icespreader"   => Some(DoorType::Icespreader  ),
            "flamethrower"  => Some(DoorType::Flamethrower ),
            "ai"            => Some(DoorType::Ai           ),
            _               => None                         ,
        }
    }

    pub const fn shield_cmdl(&self) -> ResId<res_id::CMDL> { // model of door, includes specification for which 128x128 texture to line door frame with
        match self {
            DoorType::Blue         =>   ResId::new(0x0734977A), // vanilla CMDL - "blueShield_v1" - door frame model
            DoorType::PowerOnly    =>   ResId::new(0x0734977A), // vanilla CMDL - "blueShield_v1" - door frame model
            DoorType::Purple       =>   ResId::new(0x33188D1B), // vanilla CMDL
            DoorType::White        =>   ResId::new(0x59649E9D), // vanilla CMDL
            DoorType::Red          =>   ResId::new(0xBBBA1EC7), // vanilla CMDL
            DoorType::Boost        =>   ResId::new(0x0734977A), // unused
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
            DoorType::VerticalBlue         =>   ResId::new(0x18D0AEE6), // vanilla horizontal CMDL (blue)
            DoorType::VerticalPowerOnly    =>   ResId::new(0x18D0AEE6), // vanilla CMDL
            DoorType::VerticalPurple       =>   ResId::new(0x095B0B93), // vanilla CMDL
            DoorType::VerticalWhite        =>   ResId::new(0xB7A8A4C9), // vanilla CMDL
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
            DoorType::Blue                 => structs::MapaObjectType::DoorNormal        as u32,
            DoorType::PowerOnly            => structs::MapaObjectType::DoorNormal        as u32,
            DoorType::Charge               => structs::MapaObjectType::DoorNormal        as u32,
            DoorType::Bomb                 => structs::MapaObjectType::DoorNormal        as u32,
            DoorType::Purple               => structs::MapaObjectType::DoorWave          as u32,
            DoorType::Wavebuster           => structs::MapaObjectType::DoorWave          as u32,
            DoorType::White                => structs::MapaObjectType::DoorIce           as u32,
            DoorType::Icespreader          => structs::MapaObjectType::DoorIce           as u32,
            DoorType::Red                  => structs::MapaObjectType::DoorPlasma        as u32,
            DoorType::Flamethrower         => structs::MapaObjectType::DoorPlasma        as u32,
            DoorType::VerticalBlue         => structs::MapaObjectType::DoorNormal        as u32,
            DoorType::VerticalPowerOnly    => structs::MapaObjectType::DoorNormal        as u32,
            DoorType::VerticalCharge       => structs::MapaObjectType::DoorNormal        as u32,
            DoorType::VerticalBomb         => structs::MapaObjectType::DoorNormal        as u32,
            DoorType::VerticalPurple       => structs::MapaObjectType::DoorWaveCeiling   as u32,
            DoorType::VerticalWavebuster   => structs::MapaObjectType::DoorWaveCeiling   as u32,
            DoorType::VerticalWhite        => structs::MapaObjectType::DoorIceCeiling    as u32,
            DoorType::VerticalIcespreader  => structs::MapaObjectType::DoorIceCeiling    as u32,
            DoorType::VerticalRed          => structs::MapaObjectType::DoorPlasmaCeiling as u32,
            DoorType::VerticalFlamethrower => structs::MapaObjectType::DoorPlasmaCeiling as u32,
            _ => structs::MapaObjectType::DoorShield as u32, // everything else is non-vanilla and thus shield
        }
    }

    pub const fn forcefield_txtr(&self) -> ResId<res_id::TXTR> { // texture to scroll across center of door for "forcefield" effect 16x16
        match self {
            DoorType::Blue         =>   ResId::new(0x8A7F3683), // vanilla TXTR - blue 16x16
            DoorType::PowerOnly    =>   ResId::new(0x8A7F3683), // vanilla TXTR
            DoorType::Purple       =>   ResId::new(0xF68DF7F1), // vanilla TXTR
            DoorType::White        =>   ResId::new(0xBE4CD99D), // vanilla TXTR
            DoorType::Red          =>   ResId::new(0xFC095F6C), // vanilla TXTR
            DoorType::Boost        =>   ResId::new(0x8A7F3683), // unused
            DoorType::PowerBomb    =>   ResId::new(0x1D588B22), // solid yellow
            DoorType::Bomb         =>   ResId::new(0xFC095F6C), // solid orange
            DoorType::Missile      =>   ResId::new(0x8344BEC8), // solid grey
            DoorType::Charge       =>   ResId::new(0x8A7F3683), // vanilla blue
            DoorType::Super        =>   ResId::new(0xD5C17775), // solid green
            DoorType::Disabled     =>   ResId::new(0x717AABCE), // void with specks
            DoorType::Wavebuster   =>   ResId::new(0xF68DF7F1), // vanilla TXTR
            DoorType::Icespreader  =>   ResId::new(0xBE4CD99D), // vanilla TXTR
            DoorType::Flamethrower =>   ResId::new(0xFC095F6C), // vanilla TXTR
            DoorType::Ai           =>   ResId::new(0x717AABCE), // void with specks

            // vertical doors use the same textures as their horizontal variants //
            DoorType::VerticalBlue         =>   DoorType::Blue.forcefield_txtr(),
            DoorType::VerticalPowerOnly    =>   DoorType::PowerOnly.forcefield_txtr(),
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

    pub fn holorim_texture(&self) -> ResId<res_id::TXTR> { // The the color applied from the rim of the door frame, specified in CMDL
        match self {
            DoorType::Blue                 =>   ResId::new(0x88ED4593), // vanilla TXTR - "blueholorim" texture [128x128]
            DoorType::PowerOnly            =>   ResId::new(0x88ED4593), // vanilla TXTR
            DoorType::Purple               =>   ResId::new(0xAB031EA9), // vanilla TXTR
            DoorType::White                =>   ResId::new(0xF6870C9F), // vanilla TXTR
            DoorType::Red                  =>   ResId::new(0x61A6945B), // vanilla TXTR
            DoorType::Boost                =>   ResId::new(0x88ED4593), // unused
            DoorType::PowerBomb            =>   custom_asset_ids::POWER_BOMB_DOOR_TXTR,
            DoorType::Bomb                 =>   custom_asset_ids::MORPH_BALL_BOMB_DOOR_TXTR,
            DoorType::Missile              =>   ResId::new(0x459582C1), // "bedroomeyesC"
            DoorType::Charge               =>   ResId::new(0xC7C8AF66), // banded blue ribbon
            DoorType::Super                =>   custom_asset_ids::SUPER_MISSILE_DOOR_TXTR,
            DoorType::Wavebuster           =>   custom_asset_ids::WAVEBUSTER_DOOR_TXTR,
            DoorType::Icespreader          =>   custom_asset_ids::ICESPREADER_DOOR_TXTR,
            DoorType::Flamethrower         =>   custom_asset_ids::FLAMETHROWER_DOOR_TXTR,
            DoorType::Disabled             =>   ResId::new(0x717AABCE), // void with specks
            DoorType::Ai                   =>   custom_asset_ids::AI_DOOR_TXTR,
            
            // vertical doors use the same textures as their horizontal variants //
            DoorType::VerticalBlue         =>   DoorType::Blue.holorim_texture(),
            DoorType::VerticalPowerOnly    =>   DoorType::PowerOnly.holorim_texture(),
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
        data.push((self.shield_cmdl().to_u32(),FourCC::from_bytes(b"CMDL")));
        data.push((self.forcefield_txtr().to_u32(),FourCC::from_bytes(b"TXTR")));
        if self.holorim_texture() != 0x00000000 {
            data.push((self.holorim_texture().to_u32(),FourCC::from_bytes(b"TXTR")));
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
            DoorType::PowerOnly,
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
            DoorType::VerticalPowerOnly,
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
            DoorType::PowerOnly => DamageVulnerability {
                power: TypeVulnerability::Normal as u32,
                ice: TypeVulnerability::Reflect as u32,
                wave: TypeVulnerability::Reflect as u32,
                plasma: TypeVulnerability::Reflect as u32,
                bomb: TypeVulnerability::Immune as u32,
                power_bomb: TypeVulnerability::Immune as u32,
                missile: TypeVulnerability::Reflect as u32,
                boost_ball: TypeVulnerability::Reflect as u32,
                phazon: TypeVulnerability::Reflect as u32,

                enemy_weapon0:TypeVulnerability::Immune as u32,
                enemy_weapon1:TypeVulnerability::Immune as u32,
                enemy_weapon2:TypeVulnerability::Immune as u32,
                enemy_weapon3:TypeVulnerability::Immune as u32,

                unknown_weapon0:TypeVulnerability::Immune as u32,
                unknown_weapon1:TypeVulnerability::Immune as u32,
                unknown_weapon2:TypeVulnerability::Immune as u32,

                charged_beams:ChargedBeams {
                    power:TypeVulnerability::Normal as u32,
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
            DoorType::VerticalPowerOnly    =>   DoorType::PowerOnly.vulnerability(),
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
            0x18D0AEE6 => Some(DoorType::VerticalBlue),
            0x095B0B93 => Some(DoorType::VerticalPurple),
            0xB7A8A4C9 => Some(DoorType::VerticalWhite),
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


impl BlastShieldType {
    pub fn from_str(string: &str) -> Option<Self> {
        match string.trim().to_lowercase().replace(" ","").replace("_", "").as_str() {
            "missile"       => Some(BlastShieldType::Missile      ),
            "missiles"      => Some(BlastShieldType::Missile      ),
            "powerbomb"     => Some(BlastShieldType::PowerBomb    ),
            "powerbombs"    => Some(BlastShieldType::PowerBomb    ),
            "super"         => Some(BlastShieldType::Super        ),
            "supermissile"  => Some(BlastShieldType::Super        ),
            "supermissiles" => Some(BlastShieldType::Super        ),
            "wavebuster"    => Some(BlastShieldType::Wavebuster   ),
            "icespreader"   => Some(BlastShieldType::Icespreader  ),
            "flamethrower"  => Some(BlastShieldType::Flamethrower ),
            _               => None                              ,
        }
    }

    pub const fn cmdl(&self) -> ResId<res_id::CMDL> {
        match self {
            BlastShieldType::PowerBomb    => custom_asset_ids::POWER_BOMB_BLAST_SHIELD_CMDL,
            BlastShieldType::Super        => custom_asset_ids::SUPER_BLAST_SHIELD_CMDL,
            BlastShieldType::Wavebuster   => custom_asset_ids::WAVEBUSTER_BLAST_SHIELD_CMDL,
            BlastShieldType::Icespreader  => custom_asset_ids::ICESPREADER_BLAST_SHIELD_CMDL,
            BlastShieldType::Flamethrower => custom_asset_ids::FLAMETHROWER_BLAST_SHIELD_CMDL,
            _ => ResId::new(0xEFDFFB8C), // Vanilla missile lock model
        }
    }

    pub const fn metal_body_txtr(&self) -> ResId<res_id::TXTR> {
        match self {
            BlastShieldType::PowerBomb    => custom_asset_ids::POWER_BOMB_BLAST_SHIELD_TXTR,
            BlastShieldType::Super        => custom_asset_ids::SUPER_BLAST_SHIELD_TXTR,
            BlastShieldType::Wavebuster   => custom_asset_ids::WAVEBUSTER_BLAST_SHIELD_TXTR,
            BlastShieldType::Icespreader  => custom_asset_ids::ICESPREADER_BLAST_SHIELD_TXTR,
            BlastShieldType::Flamethrower => custom_asset_ids::FLAMETHROWER_BLAST_SHIELD_TXTR,
            _ => ResId::new(0x6E09EA6B), // Vanilla missile lock txtr
        }
    }

    pub const fn glow_border_txtr(&self) -> ResId<res_id::TXTR> {
        match self {
            BlastShieldType::PowerBomb    => custom_asset_ids::BLAST_SHIELD_ALT_TXTR0,
            BlastShieldType::Super        => custom_asset_ids::BLAST_SHIELD_ALT_TXTR0,
            BlastShieldType::Wavebuster   => custom_asset_ids::BLAST_SHIELD_ALT_TXTR0,
            BlastShieldType::Icespreader  => custom_asset_ids::BLAST_SHIELD_ALT_TXTR0,
            BlastShieldType::Flamethrower => custom_asset_ids::BLAST_SHIELD_ALT_TXTR0,
            _ => ResId::new(0x5B97098E), // Vanilla missile lock txtr
        }
    }

    pub const fn glow_trim_txtr(&self) -> ResId<res_id::TXTR> {
        match self {
            BlastShieldType::PowerBomb    => custom_asset_ids::BLAST_SHIELD_ALT_TXTR1,
            BlastShieldType::Super        => custom_asset_ids::BLAST_SHIELD_ALT_TXTR1,
            BlastShieldType::Wavebuster   => custom_asset_ids::BLAST_SHIELD_ALT_TXTR1,
            BlastShieldType::Icespreader  => custom_asset_ids::BLAST_SHIELD_ALT_TXTR1,
            BlastShieldType::Flamethrower => custom_asset_ids::BLAST_SHIELD_ALT_TXTR1,
            _ => ResId::new(0x5C7B215C), // Vanilla missile lock txtr
        }
    }

    pub const fn animated_glow_txtr(&self) -> ResId<res_id::TXTR> {
        match self {
            BlastShieldType::PowerBomb    => custom_asset_ids::BLAST_SHIELD_ALT_TXTR2,
            BlastShieldType::Super        => custom_asset_ids::BLAST_SHIELD_ALT_TXTR2,
            BlastShieldType::Wavebuster   => custom_asset_ids::BLAST_SHIELD_ALT_TXTR2,
            BlastShieldType::Icespreader  => custom_asset_ids::BLAST_SHIELD_ALT_TXTR2,
            BlastShieldType::Flamethrower => custom_asset_ids::BLAST_SHIELD_ALT_TXTR2,
            _ => ResId::new(0xFA0C2AE8), // Vanilla missile lock txtrw
        }
    }
    
    pub const fn metal_trim_txtr(&self) -> ResId<res_id::TXTR> {
        match self {
            _ => ResId::new(0xFDE0023A), // Vanilla missile lock txtr
        }
    }

    pub const fn scan(&self) -> ResId<res_id::SCAN> {
        match self {
            BlastShieldType::PowerBomb    => custom_asset_ids::POWER_BOMB_BLAST_SHIELD_SCAN,
            BlastShieldType::Super        => custom_asset_ids::SUPER_BLAST_SHIELD_SCAN,
            BlastShieldType::Wavebuster   => custom_asset_ids::WAVEBUSTER_BLAST_SHIELD_SCAN,
            BlastShieldType::Icespreader  => custom_asset_ids::ICESPREADER_BLAST_SHIELD_SCAN,
            BlastShieldType::Flamethrower => custom_asset_ids::FLAMETHROWER_BLAST_SHIELD_SCAN,
            _ => ResId::invalid(), // Vanilla missile locks do not have scans associated with the actor
        }
    }

    pub const fn strg(&self) -> ResId<res_id::STRG> {
        match self {
            BlastShieldType::PowerBomb    => custom_asset_ids::POWER_BOMB_BLAST_SHIELD_STRG,
            BlastShieldType::Super        => custom_asset_ids::SUPER_BLAST_SHIELD_STRG,
            BlastShieldType::Wavebuster   => custom_asset_ids::WAVEBUSTER_BLAST_SHIELD_STRG,
            BlastShieldType::Icespreader  => custom_asset_ids::ICESPREADER_BLAST_SHIELD_STRG,
            BlastShieldType::Flamethrower => custom_asset_ids::FLAMETHROWER_BLAST_SHIELD_STRG,
            _ => ResId::invalid(), // Vanilla missile locks do not have scans associated with the actor
        }
    }

    pub fn scan_text(&self) -> Vec<String> {
        match self {
            BlastShieldType::PowerBomb    => vec!["There is an Advanced Blast Shield on the door blocking access. Analysis indicates that the Blast Shield is reinforced\0".to_string(),
                                                  "\0".to_string(),
                                                  "with &push;&main-color=#D91818;Bendenzium&pop;, rendering it invulnerable to most weapons.\0".to_string(),
                                                 ],
            BlastShieldType::Super        => vec!["There is an Advanced Blast Shield on the door blocking access. Analysis indicates that the Blast Shield is reinforced\0".to_string(),
                                                  "\0".to_string(),
                                                  "with &push;&main-color=#D91818;Cordite&pop;, rendering it invulnerable to most weapons.\0".to_string(),
                                                 ],
            BlastShieldType::Wavebuster   => vec!["There is an Elemental Blast Shield on the door blocking access. Analysis indicates that the Blast Shield is invulnerable\0".to_string(),
                                                  "\0".to_string(),
                                                  "to standard Beam fire. Continuous exposure to &push;&main-color=#D91818;Extreme Amperage&pop; may damage it.\0".to_string(),
                                                 ],
            BlastShieldType::Icespreader  => vec!["There is an Elemental Blast Shield on the door blocking access. Analysis indicates that the Blast Shield is invulnerable\0".to_string(),
                                                  "\0".to_string(),
                                                  "to standard Beam fire. A concussive blast augmented with &push;&main-color=#D91818;Extreme Cold&pop; may damage it.\0".to_string(),
                                                 ],
            BlastShieldType::Flamethrower => vec!["There is an Elemental Blast Shield on the door blocking access. Analysis indicates that the Blast Shield is invulnerable\0".to_string(),
                                                  "\0".to_string(),
                                                  "to standard Beam fire. Continuous exposure to &push;&main-color=#D91818;Extreme Heat&pop; may damage it.\0".to_string(),
                                                 ],
            _ => vec!["\0".to_string()], // Vanilla missile locks do not have scans associated with the actor
        }
    }

    pub fn dependencies(&self) -> Vec<(u32, FourCC)> { // dependencies to add to the area
        
        let mut data: Vec<(u32, FourCC)> = Vec::new();
        data.push((self.cmdl().to_u32(),               FourCC::from_bytes(b"CMDL")));
        data.push((self.metal_body_txtr().to_u32(),    FourCC::from_bytes(b"TXTR")));
        data.push((self.glow_border_txtr().to_u32(),   FourCC::from_bytes(b"TXTR")));
        data.push((self.glow_trim_txtr().to_u32(),     FourCC::from_bytes(b"TXTR")));
        data.push((self.animated_glow_txtr().to_u32(), FourCC::from_bytes(b"TXTR")));
        data.push((self.metal_trim_txtr().to_u32(),    FourCC::from_bytes(b"TXTR")));
        data.push((self.scan().to_u32(),               FourCC::from_bytes(b"SCAN")));
        data.push((self.strg().to_u32(),               FourCC::from_bytes(b"STRG")));
        data.retain(|i| i.0 != 0xffffffff && i.0 != 0);
        data
    }

    pub fn iter() -> impl Iterator<Item = BlastShieldType> {
        [
            BlastShieldType::Missile,
            BlastShieldType::PowerBomb,
            BlastShieldType::Super,
            BlastShieldType::Wavebuster,
            BlastShieldType::Icespreader,
            BlastShieldType::Flamethrower,
        ].iter().map(|i| *i)
    }

    pub fn vulnerability(&self) -> DamageVulnerability { // just re-use the door vulnerabilites
        match self {
            BlastShieldType::Missile        => DoorType::Missile.vulnerability(),
            BlastShieldType::PowerBomb      => DoorType::PowerBomb.vulnerability(),
            BlastShieldType::Super          => DoorType::Super.vulnerability(),
            BlastShieldType::Wavebuster     => DoorType::Wavebuster.vulnerability(),
            BlastShieldType::Icespreader    => DoorType::Icespreader.vulnerability(),
            BlastShieldType::Flamethrower   => DoorType::Flamethrower.vulnerability(),
            
        }
    }

    pub const fn door_type_counterpart(&self) -> DoorType {
        match self {
            BlastShieldType::Missile        => DoorType::Missile,
            BlastShieldType::PowerBomb      => DoorType::PowerBomb,
            BlastShieldType::Super          => DoorType::Super,
            BlastShieldType::Wavebuster     => DoorType::Wavebuster,
            BlastShieldType::Icespreader    => DoorType::Icespreader,
            BlastShieldType::Flamethrower   => DoorType::Flamethrower,
        }
    }
}
