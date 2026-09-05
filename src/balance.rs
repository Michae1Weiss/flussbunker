//! Every number a designer would want to change lives here, and nowhere else.
//!
//! Balancing *is* tower-defense design, and it happens hundreds of times. A
//! magic number inside a system is a number that can only be found by reading
//! code. Keeping them in one table means a balance pass is one file diff.
//!
//! Deliberately **not** RON yet. A const table costs zero infrastructure and is
//! type-checked. Promote to `assets/balance.ron` the day you start changing
//! numbers more often than code — that trigger has not fired yet.
//!
//! Note the types: plain `f32`/`u32`/`&str`, no `Color`, no `Timer`, no
//! `Handle`. Data stays engine-agnostic; the systems turn it into Bevy types.
//! That is what makes this file readable to someone who does not know Bevy.

use crate::game::combat::DamageKind;

// --- World layout ---------------------------------------------------------

pub const TILE_SIZE: f32 = 48.0;

// Draw order. Named constants beat scattered `0.5`s in `Vec3::new(x, y, ?)`.
pub const Z_TILE: f32 = 0.0;
pub const Z_TOWER: f32 = 10.0;
pub const Z_UBOAT: f32 = 15.0;
pub const Z_ENEMY: f32 = 20.0;
pub const Z_PROJECTILE: f32 = 30.0;

// --- Run setup ------------------------------------------------------------

pub const STARTING_MARKS: u32 = 260;
pub const STARTING_INTEGRITY: u32 = 20;

// --- Towers ---------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TowerKind {
    /// 2 cm Flak 38 — fast, cheap, chews through unarmoured hulls.
    Flak,
    /// 15 cm coastal gun — slow, expensive, hits hard but still a shell.
    Coastal,
}

pub struct TowerStats {
    pub name: &'static str,
    pub cost: u32,
    pub range: f32,
    pub damage: f32,
    /// Seconds between shots.
    pub reload: f32,
    pub projectile_speed: f32,
    pub damage_kind: DamageKind,
    pub rgb: [f32; 3],
}

impl TowerKind {
    pub const ALL: [TowerKind; 2] = [TowerKind::Flak, TowerKind::Coastal];

    pub const fn stats(self) -> TowerStats {
        match self {
            TowerKind::Flak => TowerStats {
                name: "Flak 38",
                cost: 60,
                range: 2.6 * TILE_SIZE,
                damage: 9.0,
                reload: 0.35,
                projectile_speed: 900.0,
                damage_kind: DamageKind::Shell,
                rgb: [0.85, 0.78, 0.45],
            },
            TowerKind::Coastal => TowerStats {
                name: "15 cm Kuestengeschuetz",
                cost: 150,
                range: 4.2 * TILE_SIZE,
                damage: 46.0,
                reload: 2.1,
                projectile_speed: 620.0,
                damage_kind: DamageKind::Shell,
                rgb: [0.80, 0.45, 0.30],
            },
        }
    }
}

// --- Enemies --------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnemyKind {
    /// Fair Mile motor torpedo boat: fast, fragile, arrives in swarms.
    MotorTorpedoBoat,
    /// Flower-class corvette: the baseline.
    Corvette,
    /// Destroyer: armoured. Shells bounce; torpedoes do not. Also the only
    /// enemy that fights back against u-boats.
    Destroyer,
}

pub struct EnemyStats {
    pub name: &'static str,
    pub health: f32,
    /// Flat reduction applied to each *shell* hit. This single number is the
    /// whole counterplay system: it makes gun-only builds fail.
    pub armor: f32,
    /// Pixels per second.
    pub speed: f32,
    pub bounty: u32,
    /// Integrity lost if it reaches the bunker.
    pub breach_damage: u32,
    pub size: f32,
    pub rgb: [f32; 3],
    /// Destroyers only: depth-charge range, damage, reload.
    pub depth_charge: Option<DepthCharge>,
}

#[derive(Clone, Copy)]
pub struct DepthCharge {
    pub range: f32,
    pub damage: f32,
    pub reload: f32,
}

impl EnemyKind {
    pub const fn stats(self) -> EnemyStats {
        match self {
            EnemyKind::MotorTorpedoBoat => EnemyStats {
                name: "MTB",
                health: 34.0,
                armor: 0.0,
                speed: 110.0,
                bounty: 9,
                breach_damage: 1,
                size: 16.0,
                rgb: [0.55, 0.75, 0.55],
                depth_charge: None,
            },
            EnemyKind::Corvette => EnemyStats {
                name: "Corvette",
                health: 110.0,
                armor: 2.0,
                speed: 62.0,
                bounty: 18,
                breach_damage: 2,
                size: 24.0,
                rgb: [0.45, 0.62, 0.72],
                depth_charge: None,
            },
            EnemyKind::Destroyer => EnemyStats {
                name: "Destroyer",
                health: 340.0,
                // 7 flat vs a Flak shell's 9 damage: guns alone barely scratch it.
                armor: 7.0,
                speed: 46.0,
                bounty: 55,
                breach_damage: 4,
                size: 32.0,
                rgb: [0.72, 0.35, 0.35],
                depth_charge: Some(DepthCharge {
                    range: 2.0 * TILE_SIZE,
                    damage: 22.0,
                    reload: 1.6,
                }),
            },
        }
    }
}

// --- U-boats --------------------------------------------------------------

pub const UBOAT_COST: u32 = 110;
pub const UBOAT_BUILD_SECONDS: f32 = 9.0;
pub const UBOAT_PEN_SLOTS: u8 = 2;
pub const UBOAT_HEALTH: f32 = 80.0;
pub const UBOAT_SPEED: f32 = 85.0;
pub const UBOAT_RANGE: f32 = 2.4 * TILE_SIZE;
pub const UBOAT_TORPEDO_DAMAGE: f32 = 60.0;
pub const UBOAT_TORPEDO_RELOAD: f32 = 2.4;
pub const UBOAT_TORPEDOES: u8 = 4;

// --- Waves ----------------------------------------------------------------

pub struct Group {
    pub kind: EnemyKind,
    pub count: u32,
    /// Seconds between two boats of this group.
    pub spacing: f32,
    /// Seconds of silence before the group starts.
    pub lead_in: f32,
}

pub struct Wave {
    pub name: &'static str,
    pub groups: &'static [Group],
}

/// Pacing intent, wave by wave — write it down, then check the numbers match:
///   1  teach: one threat, no pressure
///   2  volume: same threat, more of it
///   3  introduce the corvette (guns still enough)
///   4  first destroyer, telegraphed and alone — "your guns are not working"
///   5  mixed: escorts force target priority, destroyer forces the u-boat
pub const WAVES: &[Wave] = &[
    Wave {
        name: "Aufklaerung",
        groups: &[Group { kind: EnemyKind::MotorTorpedoBoat, count: 6, spacing: 1.1, lead_in: 0.0 }],
    },
    Wave {
        name: "Schnellbootrudel",
        groups: &[Group { kind: EnemyKind::MotorTorpedoBoat, count: 12, spacing: 0.6, lead_in: 0.0 }],
    },
    Wave {
        name: "Geleitzug",
        groups: &[
            Group { kind: EnemyKind::Corvette, count: 4, spacing: 1.8, lead_in: 0.0 },
            Group { kind: EnemyKind::MotorTorpedoBoat, count: 8, spacing: 0.5, lead_in: 6.0 },
        ],
    },
    Wave {
        name: "Zerstoerer voraus",
        groups: &[
            Group { kind: EnemyKind::Destroyer, count: 1, spacing: 0.0, lead_in: 0.0 },
            Group { kind: EnemyKind::Corvette, count: 5, spacing: 1.4, lead_in: 5.0 },
        ],
    },
    Wave {
        name: "Grossangriff",
        groups: &[
            Group { kind: EnemyKind::Destroyer, count: 3, spacing: 4.0, lead_in: 0.0 },
            Group { kind: EnemyKind::Corvette, count: 6, spacing: 1.2, lead_in: 3.0 },
            Group { kind: EnemyKind::MotorTorpedoBoat, count: 14, spacing: 0.4, lead_in: 8.0 },
        ],
    },
];
