//! Every number a designer would want to change lives here, and nowhere else.
//!
//! v2 note: the whole first playtest was fixed in this file plus `uboat.rs`.
//! That is the payoff for keeping tuning out of systems — "the game is too
//! easy and too short" turned out to be a table edit, not a refactor.

use crate::game::combat::DamageKind;

// --- World layout ---------------------------------------------------------

pub const TILE_SIZE: f32 = 48.0;

pub const Z_TILE: f32 = 0.0;
pub const Z_TOWER: f32 = 10.0;
pub const Z_UBOAT: f32 = 15.0;
pub const Z_ENEMY: f32 = 20.0;
pub const Z_PROJECTILE: f32 = 30.0;

// --- Run setup ------------------------------------------------------------

/// Tightened from 260. You never lost, and a big opening float is the usual
/// reason: it lets the first three waves be solved by overbuilding.
pub const STARTING_MARKS: u32 = 170;
pub const STARTING_INTEGRITY: u32 = 20;

// --- Towers ---------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TowerKind {
    /// 2 cm Flak 38 — cheap, fast, single target.
    Flak,
    /// 8 cm Granatwerfer — splash. The answer to swarms.
    Mortar,
    /// 15 cm Kuestengeschuetz — slow, expensive, heavy single hit.
    Coastal,
}

pub struct TowerStats {
    pub name: &'static str,
    /// Shown on the build bar. The playtest said the costs were invisible.
    pub hotkey: &'static str,
    pub cost: u32,
    pub range: f32,
    pub damage: f32,
    pub reload: f32,
    pub projectile_speed: f32,
    /// Radius of area damage on impact, if any.
    pub splash: f32,
    pub damage_kind: DamageKind,
    pub rgb: [f32; 3],
}

impl TowerKind {
    pub const ALL: [TowerKind; 3] = [TowerKind::Flak, TowerKind::Mortar, TowerKind::Coastal];

    pub const fn stats(self) -> TowerStats {
        match self {
            TowerKind::Flak => TowerStats {
                name: "Flak 38",
                hotkey: "1",
                cost: 60,
                range: 2.6 * TILE_SIZE,
                damage: 9.0,
                reload: 0.35,
                projectile_speed: 900.0,
                splash: 0.0,
                damage_kind: DamageKind::Shell,
                rgb: [0.85, 0.78, 0.45],
            },
            TowerKind::Mortar => TowerStats {
                name: "Granatwerfer",
                hotkey: "2",
                cost: 115,
                range: 3.4 * TILE_SIZE,
                damage: 15.0,
                reload: 1.5,
                projectile_speed: 420.0,
                splash: 44.0,
                damage_kind: DamageKind::Shell,
                rgb: [0.55, 0.70, 0.50],
            },
            TowerKind::Coastal => TowerStats {
                name: "15 cm Kuestengeschuetz",
                hotkey: "3",
                cost: 175,
                range: 4.4 * TILE_SIZE,
                damage: 52.0,
                reload: 2.2,
                projectile_speed: 620.0,
                splash: 0.0,
                damage_kind: DamageKind::Shell,
                rgb: [0.80, 0.45, 0.30],
            },
        }
    }
}

// --- Enemies --------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnemyKind {
    MotorTorpedoBoat,
    Corvette,
    /// Landing craft: slow, tough, and a breach hurts badly. Punishes leaks
    /// rather than defences — the enemy that makes you lose on wave 9.
    LandingCraft,
    /// The only thing that can sink your u-boat.
    Destroyer,
    /// Monitor: 12 armour. Shells are almost literally useless. Wave 12.
    Monitor,
}

pub struct EnemyStats {
    pub name: &'static str,
    pub health: f32,
    pub armor: f32,
    pub speed: f32,
    pub bounty: u32,
    pub breach_damage: u32,
    pub size: f32,
    pub rgb: [f32; 3],
    pub depth_charge: Option<DepthCharge>,
}

#[derive(Clone, Copy)]
pub struct DepthCharge {
    pub range: f32,
    pub damage: f32,
    /// Seconds of visible sonar contact before the charges drop. This is the
    /// warning window — the entire reason losing the boat can feel fair.
    pub acquire: f32,
}

impl EnemyKind {
    pub const fn stats(self) -> EnemyStats {
        match self {
            EnemyKind::MotorTorpedoBoat => EnemyStats {
                name: "MTB",
                health: 38.0,
                armor: 0.0,
                speed: 118.0,
                bounty: 9,
                breach_damage: 1,
                size: 16.0,
                rgb: [0.55, 0.75, 0.55],
                depth_charge: None,
            },
            EnemyKind::Corvette => EnemyStats {
                name: "Korvette",
                health: 130.0,
                armor: 3.0,
                speed: 62.0,
                bounty: 17,
                breach_damage: 2,
                size: 24.0,
                rgb: [0.45, 0.62, 0.72],
                depth_charge: None,
            },
            EnemyKind::LandingCraft => EnemyStats {
                name: "Landungsboot",
                health: 300.0,
                armor: 1.0,
                speed: 38.0,
                bounty: 30,
                breach_damage: 5,
                size: 30.0,
                rgb: [0.62, 0.58, 0.40],
                depth_charge: None,
            },
            EnemyKind::Destroyer => EnemyStats {
                name: "Zerstoerer",
                health: 380.0,
                armor: 8.0,
                speed: 50.0,
                bounty: 55,
                breach_damage: 4,
                size: 32.0,
                rgb: [0.72, 0.35, 0.35],
                depth_charge: Some(DepthCharge {
                    range: 2.6 * TILE_SIZE,
                    damage: 55.0,
                    acquire: 1.9,
                }),
            },
            EnemyKind::Monitor => EnemyStats {
                name: "Monitor",
                health: 1100.0,
                // vs a 52-damage coastal gun that is 44 per hit; vs Flak it is
                // the 10% floor. Torpedoes or nothing.
                armor: 12.0,
                speed: 30.0,
                bounty: 120,
                breach_damage: 8,
                size: 40.0,
                rgb: [0.80, 0.30, 0.45],
                depth_charge: Some(DepthCharge {
                    range: 2.2 * TILE_SIZE,
                    damage: 45.0,
                    acquire: 2.4,
                }),
            },
        }
    }
}

// --- U-boat ---------------------------------------------------------------

/// You start with one, free. Losing it and buying another is the punishment.
pub const UBOAT_REPLACEMENT_COST: u32 = 220;
pub const UBOAT_HEALTH: f32 = 130.0;
/// Was 85. Travel time was most of the mechanic's runtime; now it is a
/// fraction of it.
pub const UBOAT_SPEED: f32 = 190.0;
/// Submerged you are safe, slower, and blind.
pub const UBOAT_SUBMERGED_SPEED_FACTOR: f32 = 0.7;
/// Dive and surface are not instant. That delay is the whole cost of diving —
/// without it, "always submerged" would be free and the choice would vanish.
pub const UBOAT_TRIM_SECONDS: f32 = 1.4;
/// Seconds per torpedo while docked at the bunker.
pub const UBOAT_REARM_SECONDS: f32 = 1.6;
/// How far a torpedo runs before it loses the water or runs out. Long, because
/// it is directional: it only threatens whatever shares your straight.
pub const TORPEDO_RUN: f32 = 9.0 * TILE_SIZE;
pub const TORPEDO_SPEED: f32 = 340.0;

/// Veterancy. Permanent, and permanently lost with the boat.
pub struct Rank {
    pub name: &'static str,
    pub kills_required: u32,
    pub torpedoes: u8,
    pub damage: f32,
    pub reload: f32,
}

pub const RANKS: &[Rank] = &[
    Rank { name: "Neuling",       kills_required: 0,  torpedoes: 4, damage: 65.0,  reload: 2.2 },
    Rank { name: "Erfahren",      kills_required: 8,  torpedoes: 5, damage: 80.0,  reload: 2.0 },
    Rank { name: "Veteran",       kills_required: 22, torpedoes: 6, damage: 100.0, reload: 1.8 },
    Rank { name: "Ritterkreuz",   kills_required: 45, torpedoes: 7, damage: 130.0, reload: 1.5 },
];

pub fn rank_for(kills: u32) -> &'static Rank {
    RANKS
        .iter()
        .rev()
        .find(|r| kills >= r.kills_required)
        .unwrap_or(&RANKS[0])
}

// --- Waves ----------------------------------------------------------------

pub struct Group {
    pub kind: EnemyKind,
    pub count: u32,
    pub spacing: f32,
    pub lead_in: f32,
}

pub struct Wave {
    pub name: &'static str,
    /// Shown during the prep phase. You had played three rounds without
    /// discovering the u-boat; a game that teaches nothing on screen deserves
    /// that outcome.
    pub hint: &'static str,
    pub groups: &'static [Group],
}

/// Twelve waves, roughly 15 minutes. Escalation is deliberate, not a curve
/// formula: every third wave introduces a threat the previous build cannot
/// answer, and the two waves after it drill that answer under pressure.
pub const WAVES: &[Wave] = &[
    Wave {
        name: "Aufklaerung",
        hint: "Linksklick auf Land baut. 1/2/3 waehlt das Geschuetz.",
        groups: &[Group { kind: EnemyKind::MotorTorpedoBoat, count: 6, spacing: 1.1, lead_in: 0.0 }],
    },
    Wave {
        name: "Schnellbootrudel",
        hint: "Linksklick auf den Fluss schickt U-96 dorthin. Torpedos laufen geradeaus.",
        groups: &[Group { kind: EnemyKind::MotorTorpedoBoat, count: 12, spacing: 0.55, lead_in: 0.0 }],
    },
    Wave {
        name: "Geleitzug",
        hint: "Stell U-96 auf eine lange Gerade. Kurven sind totes Wasser.",
        groups: &[
            Group { kind: EnemyKind::Corvette, count: 4, spacing: 1.8, lead_in: 0.0 },
            Group { kind: EnemyKind::MotorTorpedoBoat, count: 8, spacing: 0.5, lead_in: 6.0 },
        ],
    },
    Wave {
        name: "Zerstoerer voraus",
        hint: "Zerstoerer sind gepanzert - Granaten prallen ab. Und sie jagen U-Boote.",
        groups: &[
            Group { kind: EnemyKind::Destroyer, count: 1, spacing: 0.0, lead_in: 0.0 },
            Group { kind: EnemyKind::Corvette, count: 5, spacing: 1.4, lead_in: 5.0 },
        ],
    },
    Wave {
        name: "Wasserbomben",
        hint: "Roter Sonarstrahl = du wirst geortet. T taucht. Getaucht bist du sicher, aber blind.",
        groups: &[
            Group { kind: EnemyKind::Destroyer, count: 2, spacing: 5.0, lead_in: 0.0 },
            Group { kind: EnemyKind::MotorTorpedoBoat, count: 10, spacing: 0.5, lead_in: 4.0 },
        ],
    },
    Wave {
        name: "Landungsversuch",
        hint: "Landungsboote sind langsam - aber ein Durchbruch kostet 5 Bunker.",
        groups: &[
            Group { kind: EnemyKind::LandingCraft, count: 3, spacing: 3.0, lead_in: 0.0 },
            Group { kind: EnemyKind::Corvette, count: 4, spacing: 1.6, lead_in: 6.0 },
        ],
    },
    Wave {
        name: "Doppelschlag",
        hint: "Granatwerfer treffen mehrere Boote auf einmal.",
        groups: &[
            Group { kind: EnemyKind::MotorTorpedoBoat, count: 18, spacing: 0.35, lead_in: 0.0 },
            Group { kind: EnemyKind::Destroyer, count: 2, spacing: 4.0, lead_in: 8.0 },
        ],
    },
    Wave {
        name: "Sperrbrecher",
        hint: "Nachladen geht nur am Bunker. Fahr rechtzeitig heim.",
        groups: &[
            Group { kind: EnemyKind::LandingCraft, count: 4, spacing: 2.4, lead_in: 0.0 },
            Group { kind: EnemyKind::Destroyer, count: 2, spacing: 6.0, lead_in: 3.0 },
            Group { kind: EnemyKind::MotorTorpedoBoat, count: 10, spacing: 0.45, lead_in: 12.0 },
        ],
    },
    Wave {
        name: "Jagdgruppe",
        hint: "Drei Zerstoerer. Getaucht anfahren, auftauchen, schiessen, wieder tauchen.",
        groups: &[
            Group { kind: EnemyKind::Destroyer, count: 3, spacing: 3.5, lead_in: 0.0 },
            Group { kind: EnemyKind::Corvette, count: 6, spacing: 1.2, lead_in: 6.0 },
        ],
    },
    Wave {
        name: "Grossangriff",
        hint: "Nichts Neues - nur alles gleichzeitig.",
        groups: &[
            Group { kind: EnemyKind::MotorTorpedoBoat, count: 20, spacing: 0.3, lead_in: 0.0 },
            Group { kind: EnemyKind::Corvette, count: 8, spacing: 1.0, lead_in: 5.0 },
            Group { kind: EnemyKind::LandingCraft, count: 4, spacing: 2.0, lead_in: 14.0 },
        ],
    },
    Wave {
        name: "Vorhut der Flotte",
        hint: "Der Monitor kommt. 12 Panzerung - Geschuetze koennen ihn nicht toeten.",
        groups: &[
            Group { kind: EnemyKind::Destroyer, count: 4, spacing: 3.0, lead_in: 0.0 },
            Group { kind: EnemyKind::LandingCraft, count: 5, spacing: 2.0, lead_in: 8.0 },
        ],
    },
    Wave {
        name: "HMS Erebus",
        hint: "Nur Torpedos. Halte die Gerade frei und lade zwischendurch nach.",
        groups: &[
            Group { kind: EnemyKind::Monitor, count: 2, spacing: 12.0, lead_in: 0.0 },
            Group { kind: EnemyKind::Destroyer, count: 4, spacing: 4.0, lead_in: 4.0 },
            Group { kind: EnemyKind::MotorTorpedoBoat, count: 16, spacing: 0.35, lead_in: 18.0 },
        ],
    },
];
