//! Every tuning number, engine-free. Ranges and speeds are world units; times are seconds.

pub const TILE_SIZE: f32 = 48.0;
/// The world is a fixed 16:9 map; the camera scales to fit it.
pub const MAP_WIDTH_TILES: f32 = 32.0;
pub const MAP_HEIGHT_TILES: f32 = 18.0;
pub const RIVER_HALF_WIDTH: f32 = 0.9 * TILE_SIZE;
/// Click and draw radius of a build spot.
pub const BUILD_SPOT_RADIUS: f32 = 0.5 * TILE_SIZE;

pub const Z_TILE: f32 = 0.0;
pub const Z_TOWER: f32 = 10.0;
pub const Z_UBOAT: f32 = 15.0;
pub const Z_ENEMY: f32 = 20.0;
pub const Z_PROJECTILE: f32 = 30.0;
pub const Z_UI: f32 = 50.0;

pub const STARTING_MARKS: u32 = 170;
pub const STARTING_INTEGRITY: u32 = 20;

/// Three roles, `TOWER_TIERS` tiers each; `stats(tier)` describes one tier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TowerKind {
    /// Naval flak: anti-swarm.
    Flak,
    /// 8.8 cm coastal gun: precise all-rounder, armour-piercing from tier 1.
    Gun88,
    /// 15 cm battery: heavy and slow, one to three barrels.
    Heavy15,
}

pub const TOWER_TIERS: u8 = 3;
/// Seconds a new turret is under construction and silent.
pub const TOWER_BUILD_SECONDS: f32 = 2.5;
/// Fraction of the RM invested in a turret returned on selling.
pub const SELL_REFUND: f32 = 0.6;
/// Armour multiplier for armour-piercing shells.
pub const AP_ARMOUR_FACTOR: f32 = 0.5;
/// Seconds the sell entry must be held.
pub const SELL_HOLD_SECONDS: f32 = 0.5;
/// World-unit distance of radial menu entries from the anchor, and their hit radius.
pub const RADIAL_RADIUS: f32 = 1.9 * TILE_SIZE;
pub const RADIAL_ENTRY_RADIUS: f32 = 0.8 * TILE_SIZE;

pub struct TowerStats {
    pub name: &'static str,
    /// RM for this tier: the build cost at tier 0, the upgrade cost above.
    pub cost: u32,
    pub range: f32,
    pub damage: f32,
    pub reload: f32,
    pub projectile_speed: f32,
    /// Shells per volley.
    pub barrels: u8,
    pub ap: bool,
    pub rgb: [f32; 3],
}

impl TowerKind {
    pub const ALL: [TowerKind; 3] = [TowerKind::Flak, TowerKind::Gun88, TowerKind::Heavy15];

    /// # Panics
    /// If `tier >= TOWER_TIERS`.
    pub const fn stats(self, tier: u8) -> TowerStats {
        assert!(tier < TOWER_TIERS);
        match self {
            TowerKind::Flak => TowerStats {
                name: "Marineflak",
                cost: [60, 50, 80][tier as usize],
                range: 2.6 * TILE_SIZE,
                damage: 9.0,
                reload: if tier == 0 { 0.35 } else { 0.28 },
                projectile_speed: 900.0,
                barrels: if tier == 2 { 2 } else { 1 },
                ap: false,
                rgb: [0.85, 0.78, 0.45],
            },
            TowerKind::Gun88 => TowerStats {
                name: "8,8 cm",
                cost: [120, 100, 150][tier as usize],
                range: 3.6 * TILE_SIZE,
                damage: 32.0,
                reload: 1.4,
                projectile_speed: 800.0,
                barrels: if tier == 2 { 2 } else { 1 },
                ap: tier >= 1,
                rgb: [0.55, 0.70, 0.50],
            },
            TowerKind::Heavy15 => TowerStats {
                name: "15 cm",
                cost: [240, 180, 280][tier as usize],
                range: 4.4 * TILE_SIZE,
                damage: 100.0,
                reload: 3.0,
                projectile_speed: 620.0,
                barrels: tier + 1,
                ap: false,
                rgb: [0.80, 0.45, 0.30],
            },
        }
    }

    /// RM spent to reach `tier` from nothing.
    pub fn total_cost(self, tier: u8) -> u32 {
        (0..=tier).map(|t| self.stats(t).cost).sum()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnemyKind {
    MotorTorpedoBoat,
    Corvette,
    LandingCraft,
    /// The only enemy that shoots at the U-boat.
    Destroyer,
    /// Belt armour above any shell's damage: torpedoes only.
    Monitor,
}

/// Drives ramming, death style and debris; see docs/design/enemies.md.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizeClass {
    Light,
    Medium,
    Heavy,
    Capital,
}

pub struct EnemyStats {
    pub name: &'static str,
    pub class: SizeClass,
    pub health: f32,
    pub armor: f32,
    pub speed: f32,
    pub bounty: u32,
    pub breach_damage: u32,
    pub size: f32,
    pub rgb: [f32; 3],
    pub shelling: Option<Shelling>,
}

/// Gunfire against a surfaced U-boat in range.
#[derive(Clone, Copy)]
pub struct Shelling {
    pub range: f32,
    pub damage: f32,
    /// Seconds of visible near misses before the first hit; resets when contact is lost.
    pub ranging: f32,
    pub reload: f32,
}

/// Minimum gap between a ship and the one ahead, beyond their hull sizes.
pub const CONVOY_GAP: f32 = 0.3 * TILE_SIZE;

impl EnemyKind {
    pub const fn stats(self) -> EnemyStats {
        match self {
            EnemyKind::MotorTorpedoBoat => EnemyStats {
                name: "MTB",
                class: SizeClass::Light,
                health: 38.0,
                armor: 0.0,
                speed: 2.45 * TILE_SIZE,
                bounty: 4,
                breach_damage: 1,
                size: 16.0,
                rgb: [0.55, 0.75, 0.55],
                shelling: None,
            },
            EnemyKind::Corvette => EnemyStats {
                name: "Korvette",
                class: SizeClass::Medium,
                health: 120.0,
                armor: 3.0,
                speed: 1.3 * TILE_SIZE,
                bounty: 12,
                breach_damage: 2,
                size: 24.0,
                rgb: [0.45, 0.62, 0.72],
                shelling: None,
            },
            EnemyKind::LandingCraft => EnemyStats {
                name: "Landungsboot",
                class: SizeClass::Medium,
                health: 300.0,
                armor: 1.0,
                speed: 0.8 * TILE_SIZE,
                bounty: 20,
                breach_damage: 5,
                size: 30.0,
                rgb: [0.62, 0.58, 0.40],
                shelling: None,
            },
            EnemyKind::Destroyer => EnemyStats {
                name: "Zerstoerer",
                class: SizeClass::Heavy,
                health: 380.0,
                armor: 12.0,
                speed: 1.05 * TILE_SIZE,
                bounty: 48,
                breach_damage: 4,
                size: 32.0,
                rgb: [0.72, 0.35, 0.35],
                shelling: Some(Shelling {
                    range: 2.6 * TILE_SIZE,
                    damage: 18.0,
                    ranging: 3.0,
                    reload: 1.5,
                }),
            },
            EnemyKind::Monitor => EnemyStats {
                name: "Monitor",
                class: SizeClass::Capital,
                health: 1200.0,
                armor: 100.0,
                speed: 0.63 * TILE_SIZE,
                bounty: 100,
                breach_damage: 8,
                size: 40.0,
                rgb: [0.80, 0.30, 0.45],
                shelling: None,
            },
        }
    }
}

pub const UBOAT_REPLACEMENT_COST: u32 = 220;
pub const UBOAT_HEALTH: f32 = 130.0;
/// Sprite length in world units; also the collision reach with ships.
pub const UBOAT_LENGTH: f32 = 30.0;
pub const UBOAT_SPEED: f32 = 190.0;
pub const UBOAT_SUBMERGED_SPEED_FACTOR: f32 = 0.7;
pub const UBOAT_TRIM_SECONDS: f32 = 1.4;
/// Radians per second; 180 degrees in about 0.8 s.
pub const UBOAT_TURN_RATE: f32 = 4.0;
/// The boat moves only once its bow is within this many radians of the travel direction.
pub const UBOAT_MOVE_ALIGNMENT: f32 = 4.0_f32.to_radians();
/// Radians per second enemy ships turn through bends without slowing.
pub const ENEMY_TURN_RATE: f32 = 3.0;
/// Radians of bow misalignment with the river axis still accepted for a shot.
pub const UBOAT_FIRE_ALIGNMENT: f32 = 10.0_f32.to_radians();
/// Seconds between tubes while Space is held.
pub const UBOAT_SALVO_SPACING: f32 = 0.3;
pub const DECK_GUN_DAMAGE: f32 = 22.0;
pub const DECK_GUN_RELOAD: f32 = 2.4;
pub const DECK_GUN_RANGE: f32 = 2.4 * TILE_SIZE;
pub const DECK_GUN_SHELL_SPEED: f32 = 700.0;
/// Fraction of the boat's max HP lost when a light boat capsizes on it.
pub const RAM_LIGHT_COST: f32 = 0.05;
/// Fraction of the boat's max HP lost when a heavier ship rams through it.
pub const RAM_HEAVY_COST: f32 = 0.40;
/// World units the boat is shoved downstream by a heavier ship.
pub const RAM_SHOVE: f32 = 1.0 * TILE_SIZE;
/// Seconds per torpedo while docked.
pub const UBOAT_REARM_SECONDS: f32 = 1.6;
/// Must be shorter than the longest straight of the river (`map::tests`).
pub const TORPEDO_RUN: f32 = 9.0 * TILE_SIZE;
pub const TORPEDO_SPEED: f32 = 340.0;

pub struct Rank {
    pub name: &'static str,
    pub kills_required: u32,
    pub torpedoes: u8,
    pub damage: f32,
    pub reload: f32,
}

pub const RANKS: &[Rank] = &[
    Rank {
        name: "Neuling",
        kills_required: 0,
        torpedoes: 4,
        damage: 65.0,
        reload: 2.2,
    },
    Rank {
        name: "Erfahren",
        kills_required: 8,
        torpedoes: 5,
        damage: 80.0,
        reload: 2.0,
    },
    Rank {
        name: "Veteran",
        kills_required: 22,
        torpedoes: 6,
        damage: 100.0,
        reload: 1.8,
    },
    Rank {
        name: "Ritterkreuz",
        kills_required: 45,
        torpedoes: 7,
        damage: 130.0,
        reload: 1.5,
    },
];

pub fn rank_for(kills: u32) -> &'static Rank {
    RANKS
        .iter()
        .rev()
        .find(|r| kills >= r.kills_required)
        .unwrap_or(&RANKS[0])
}

pub struct Group {
    pub kind: EnemyKind,
    pub count: u32,
    /// Seconds between boats of this group.
    pub spacing: f32,
    /// Seconds after the wave start before the first boat.
    pub lead_in: f32,
}

pub struct Wave {
    pub name: &'static str,
    /// Shown in the HUD during the prep phase.
    pub hint: &'static str,
    pub groups: &'static [Group],
}

pub const WAVES: &[Wave] = &[
    Wave {
        name: "Aufklaerung",
        hint: "Linksklick auf Land baut. 1/2/3 waehlt das Geschuetz.",
        groups: &[Group {
            kind: EnemyKind::MotorTorpedoBoat,
            count: 6,
            spacing: 1.1,
            lead_in: 0.0,
        }],
    },
    Wave {
        name: "Schnellbootrudel",
        hint: "Linksklick auf den Fluss schickt U-96 dorthin. Torpedos laufen geradeaus.",
        groups: &[Group {
            kind: EnemyKind::MotorTorpedoBoat,
            count: 12,
            spacing: 0.55,
            lead_in: 0.0,
        }],
    },
    Wave {
        name: "Geleitzug",
        hint: "Stell U-96 auf eine lange Gerade. Kurven sind totes Wasser.",
        groups: &[
            Group {
                kind: EnemyKind::Corvette,
                count: 4,
                spacing: 1.8,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::MotorTorpedoBoat,
                count: 8,
                spacing: 0.5,
                lead_in: 6.0,
            },
        ],
    },
    Wave {
        name: "Zerstoerer voraus",
        hint: "Zerstoerer sind gepanzert - Granaten prallen ab. Und sie jagen U-Boote.",
        groups: &[
            Group {
                kind: EnemyKind::Destroyer,
                count: 1,
                spacing: 0.0,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::Corvette,
                count: 5,
                spacing: 1.4,
                lead_in: 5.0,
            },
        ],
    },
    Wave {
        name: "Wasserbomben",
        hint:
            "Roter Sonarstrahl = du wirst geortet. T taucht. Getaucht bist du sicher, aber blind.",
        groups: &[
            Group {
                kind: EnemyKind::Destroyer,
                count: 2,
                spacing: 5.0,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::MotorTorpedoBoat,
                count: 10,
                spacing: 0.5,
                lead_in: 4.0,
            },
        ],
    },
    Wave {
        name: "Landungsversuch",
        hint: "Landungsboote sind langsam - aber ein Durchbruch kostet 5 Bunker.",
        groups: &[
            Group {
                kind: EnemyKind::LandingCraft,
                count: 3,
                spacing: 3.0,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::Corvette,
                count: 4,
                spacing: 1.6,
                lead_in: 6.0,
            },
        ],
    },
    Wave {
        name: "Doppelschlag",
        hint: "Granatwerfer treffen mehrere Boote auf einmal.",
        groups: &[
            Group {
                kind: EnemyKind::MotorTorpedoBoat,
                count: 18,
                spacing: 0.35,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::Destroyer,
                count: 2,
                spacing: 4.0,
                lead_in: 8.0,
            },
        ],
    },
    Wave {
        name: "Sperrbrecher",
        hint: "Nachladen geht nur am Bunker. Fahr rechtzeitig heim.",
        groups: &[
            Group {
                kind: EnemyKind::LandingCraft,
                count: 4,
                spacing: 2.4,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::Destroyer,
                count: 2,
                spacing: 6.0,
                lead_in: 3.0,
            },
            Group {
                kind: EnemyKind::MotorTorpedoBoat,
                count: 10,
                spacing: 0.45,
                lead_in: 12.0,
            },
        ],
    },
    Wave {
        name: "Jagdgruppe",
        hint: "Drei Zerstoerer. Getaucht anfahren, auftauchen, schiessen, wieder tauchen.",
        groups: &[
            Group {
                kind: EnemyKind::Destroyer,
                count: 3,
                spacing: 3.5,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::Corvette,
                count: 6,
                spacing: 1.2,
                lead_in: 6.0,
            },
        ],
    },
    Wave {
        name: "Grossangriff",
        hint: "Nichts Neues - nur alles gleichzeitig.",
        groups: &[
            Group {
                kind: EnemyKind::MotorTorpedoBoat,
                count: 20,
                spacing: 0.3,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::Corvette,
                count: 8,
                spacing: 1.0,
                lead_in: 5.0,
            },
            Group {
                kind: EnemyKind::LandingCraft,
                count: 4,
                spacing: 2.0,
                lead_in: 14.0,
            },
        ],
    },
    Wave {
        name: "Vorhut der Flotte",
        hint: "Der Monitor kommt. 12 Panzerung - Geschuetze koennen ihn nicht toeten.",
        groups: &[
            Group {
                kind: EnemyKind::Destroyer,
                count: 4,
                spacing: 3.0,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::LandingCraft,
                count: 5,
                spacing: 2.0,
                lead_in: 8.0,
            },
        ],
    },
    Wave {
        name: "HMS Erebus",
        hint: "Nur Torpedos. Halte die Gerade frei und lade zwischendurch nach.",
        groups: &[
            Group {
                kind: EnemyKind::Monitor,
                count: 2,
                spacing: 12.0,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::Destroyer,
                count: 4,
                spacing: 4.0,
                lead_in: 4.0,
            },
            Group {
                kind: EnemyKind::MotorTorpedoBoat,
                count: 16,
                spacing: 0.35,
                lead_in: 18.0,
            },
        ],
    },
];
