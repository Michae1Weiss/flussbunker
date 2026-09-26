//! Every tuning number, engine-free. Ranges and speeds are world units; times are seconds.

pub const TILE_SIZE: f32 = 48.0;
/// The world is a fixed 16:9 map; the camera scales to fit it.
pub const MAP_WIDTH_TILES: f32 = 32.0;
pub const MAP_HEIGHT_TILES: f32 = 18.0;
/// Paper drawn around the playfield on each side, for screens wider or taller than 16:9.
pub const MAP_BLEED_TILES: f32 = 1.0;
pub const RIVER_HALF_WIDTH: f32 = 0.9 * TILE_SIZE;
/// Click and draw radius of a build spot.
pub const BUILD_SPOT_RADIUS: f32 = 0.5 * TILE_SIZE;

pub const Z_TILE: f32 = 0.0;
pub const Z_WRECK: f32 = 1.0;
pub const Z_TOWER: f32 = 10.0;
pub const Z_UBOAT: f32 = 15.0;
pub const Z_ENEMY: f32 = 20.0;
pub const Z_PROJECTILE: f32 = 30.0;
pub const Z_DEBRIS: f32 = 40.0;
pub const Z_UI: f32 = 50.0;
/// Notes sit above the radial menu, whose entries stack a little over `Z_UI`.
pub const Z_NOTE: f32 = 60.0;
/// Seconds an onboarding note stays readable.
pub const NOTE_SECONDS: f32 = 4.0;
/// Seconds an enemy stays white after a hit.
pub const FLASH_SECONDS: f32 = 0.1;
/// Radians per second a turret barrel turns towards its target. Cosmetic: firing never waits.
pub const BARREL_TURN_RATE: f32 = 9.0;
/// World units a barrel kicks back on a shot, and seconds it takes to run out again.
pub const RECOIL_DISTANCE: f32 = 3.5;
pub const RECOIL_SECONDS: f32 = 0.15;
/// Camera offset in world units at full trauma, and trauma lost per second.
pub const SHAKE_MAX_OFFSET: f32 = 14.0;
pub const SHAKE_DECAY: f32 = 1.6;
/// Seconds the displayed RM take to catch up with the treasury.
pub const RM_TWEEN_SECONDS: f32 = 0.3;

pub const STARTING_MARKS: u32 = 0;
/// RM paid at the end of every wave, kills or not.
pub const WAVE_CLEAR_BONUS: u32 = 30;
/// Bounty of the prologue's gold transport: two flak, or one flak plus savings.
pub const PROLOGUE_BOUNTY: u32 = 150;
/// The gold transport cruises between these fractions of the river.
pub const PROLOGUE_CRUISE: (f32, f32) = (0.2, 0.75);
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
    /// Prologue target: cruises the river, never breaches, cannot shoot back.
    GoldTransport,
}

/// Drives ramming, death style and debris; see docs/design/enemies.md.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizeClass {
    Light,
    Medium,
    Heavy,
    Capital,
}

impl SizeClass {
    /// Index into the per-class tables below.
    pub fn index(self) -> usize {
        self as usize
    }
}

/// Blast radius and damage of a breaking ship, by `SizeClass`.
pub const BLAST_RADIUS: [f32; 4] = [
    1.0 * TILE_SIZE,
    1.5 * TILE_SIZE,
    2.0 * TILE_SIZE,
    3.0 * TILE_SIZE,
];
pub const BLAST_DAMAGE: [f32; 4] = [30.0, 60.0, 120.0, 250.0];
/// Seconds between one explosion and the next link of a chain.
pub const CHAIN_DELAY: f32 = 0.2;
/// Debris pieces (low, high) of a breaking ship, by `SizeClass`.
pub const DEBRIS_COUNT: [(u32, u32); 4] = [(1, 2), (3, 4), (5, 6), (8, 10)];
/// Airborne pieces at once, for readability.
pub const DEBRIS_CAP: usize = 20;
/// (damage, size) per debris type; the first is the turret and flies first.
pub const DEBRIS_TYPES: [(f32, f32); 6] = [
    (60.0, 14.0),
    (25.0, 8.0),
    (20.0, 10.0),
    (10.0, 6.0),
    (30.0, 7.0),
    (10.0, 6.0),
];
/// World units per second squared for debris arcs.
pub const DEBRIS_GRAVITY: f32 = 6.0 * TILE_SIZE;
pub const WRECK_DRIFT_SPEED: f32 = 0.6 * TILE_SIZE;
/// Seconds a wreck or crater takes to fade out once it sinks.
pub const SINK_SECONDS: f32 = 3.0;
/// Stranded wrecks and craters kept on the map; the oldest beyond this sink.
pub const WRECK_CAP: usize = 10;
pub const CRATER_CAP: usize = 24;

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
                size: 29.0,
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
                size: 43.0,
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
                size: 54.0,
                rgb: [0.62, 0.58, 0.40],
                shelling: None,
            },
            EnemyKind::Destroyer => EnemyStats {
                name: "Zerstörer",
                class: SizeClass::Heavy,
                health: 380.0,
                armor: 12.0,
                speed: 1.05 * TILE_SIZE,
                bounty: 48,
                breach_damage: 4,
                size: 58.0,
                rgb: [0.72, 0.35, 0.35],
                shelling: Some(Shelling {
                    range: 2.6 * TILE_SIZE,
                    damage: 18.0,
                    ranging: 3.0,
                    reload: 1.5,
                }),
            },
            EnemyKind::GoldTransport => EnemyStats {
                name: "Goldtransport",
                class: SizeClass::Medium,
                health: 200.0,
                armor: 0.0,
                speed: 0.7 * TILE_SIZE,
                bounty: PROLOGUE_BOUNTY,
                breach_damage: 0,
                size: 50.0,
                rgb: [0.90, 0.75, 0.30],
                shelling: None,
            },
            EnemyKind::Monitor => EnemyStats {
                name: "Monitor",
                class: SizeClass::Capital,
                health: 1200.0,
                armor: 100.0,
                speed: 0.63 * TILE_SIZE,
                bounty: 100,
                breach_damage: 8,
                size: 72.0,
                rgb: [0.80, 0.30, 0.45],
                shelling: None,
            },
        }
    }
}

/// Sprite length of the smallest model in world units; also the collision reach with ships.
pub const UBOAT_LENGTH: f32 = 54.0;
/// Added per model step: VIIA, VIIC, IXC are 54, 61, 68 long.
pub const UBOAT_LENGTH_STEP: f32 = 7.0;
pub const UBOAT_BEAM: f32 = 22.0;
/// Ship sprite width as a fraction of its hull length.
pub const HULL_BEAM_RATIO: f32 = 0.55;
/// Emplacement sprite side per tier, world units.
pub const TOWER_SIZE: [f32; 3] = [36.0, 43.2, 50.4];
pub const UBOAT_SUBMERGED_SPEED_FACTOR: f32 = 0.7;
/// Fraction of the boat's nominal value billed when it returns after sinking.
pub const REPAIR_BILL: f32 = 0.3;
/// HP fraction the boat returns with when the bill cannot be paid.
pub const UNPAID_RETURN_HP: f32 = 0.5;
/// Max-HP fraction repaired per second while docked.
pub const DOCK_REPAIR_PER_SECOND: f32 = 0.1;

/// Three linear models, bought at the docked boat; each changes behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BoatModel {
    ViiA,
    ViiC,
    IxC,
}

pub struct BoatStats {
    pub name: &'static str,
    /// RM to buy this model from the previous one.
    pub price: u32,
    /// Value the repair bill is based on.
    pub nominal: u32,
    /// First wave index (0-based) at which it can be bought.
    pub unlock_wave: usize,
    pub hp: f32,
    pub tubes: u8,
    pub torpedo_damage: f32,
    pub speed: f32,
    pub dive_seconds: f32,
}

impl BoatModel {
    pub const fn stats(self) -> BoatStats {
        match self {
            BoatModel::ViiA => BoatStats {
                name: "Typ VIIA",
                price: 0,
                nominal: 300,
                unlock_wave: 0,
                hp: 130.0,
                tubes: 4,
                torpedo_damage: 130.0,
                speed: 190.0,
                dive_seconds: 1.4,
            },
            BoatModel::ViiC => BoatStats {
                name: "Typ VIIC",
                price: 500,
                nominal: 800,
                unlock_wave: 3,
                hp: 165.0,
                tubes: 6,
                torpedo_damage: 150.0,
                speed: 195.0,
                dive_seconds: 1.1,
            },
            BoatModel::IxC => BoatStats {
                name: "Typ IXC",
                price: 900,
                nominal: 1700,
                unlock_wave: 7,
                hp: 220.0,
                tubes: 8,
                torpedo_damage: 150.0,
                speed: 170.0,
                dive_seconds: 1.8,
            },
        }
    }

    pub const fn next(self) -> Option<BoatModel> {
        match self {
            BoatModel::ViiA => Some(BoatModel::ViiC),
            BoatModel::ViiC => Some(BoatModel::IxC),
            BoatModel::IxC => None,
        }
    }
}
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
    pub groups: &'static [Group],
}

/// Twelve waves from docs/balance: fast groups first so slow ships form the tail of the convoy.
pub const WAVES: &[Wave] = &[
    Wave {
        name: "Aufklärung",
        groups: &[Group {
            kind: EnemyKind::MotorTorpedoBoat,
            count: 6,
            spacing: 0.5,
            lead_in: 0.0,
        }],
    },
    Wave {
        name: "Schnellbootrudel",
        groups: &[Group {
            kind: EnemyKind::MotorTorpedoBoat,
            count: 12,
            spacing: 0.5,
            lead_in: 0.0,
        }],
    },
    Wave {
        name: "Geleitzug",
        groups: &[
            Group {
                kind: EnemyKind::MotorTorpedoBoat,
                count: 8,
                spacing: 0.5,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::Corvette,
                count: 4,
                spacing: 1.6,
                lead_in: 4.1,
            },
        ],
    },
    Wave {
        name: "Zerstörer voraus",
        groups: &[
            Group {
                kind: EnemyKind::MotorTorpedoBoat,
                count: 4,
                spacing: 0.5,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::Corvette,
                count: 3,
                spacing: 1.6,
                lead_in: 2.9,
            },
            Group {
                kind: EnemyKind::Destroyer,
                count: 1,
                spacing: 4.0,
                lead_in: 6.8,
            },
        ],
    },
    Wave {
        name: "Unter Beschuss",
        groups: &[
            Group {
                kind: EnemyKind::MotorTorpedoBoat,
                count: 10,
                spacing: 0.5,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::Destroyer,
                count: 2,
                spacing: 4.0,
                lead_in: 4.7,
            },
        ],
    },
    Wave {
        name: "Landung",
        groups: &[
            Group {
                kind: EnemyKind::MotorTorpedoBoat,
                count: 6,
                spacing: 0.5,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::Corvette,
                count: 4,
                spacing: 1.6,
                lead_in: 3.5,
            },
            Group {
                kind: EnemyKind::LandingCraft,
                count: 3,
                spacing: 2.5,
                lead_in: 8.4,
            },
        ],
    },
    Wave {
        name: "Doppelschlag",
        groups: &[
            Group {
                kind: EnemyKind::MotorTorpedoBoat,
                count: 18,
                spacing: 0.5,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::Destroyer,
                count: 2,
                spacing: 4.0,
                lead_in: 7.1,
            },
        ],
    },
    Wave {
        name: "Sperrbrecher",
        groups: &[
            Group {
                kind: EnemyKind::MotorTorpedoBoat,
                count: 10,
                spacing: 0.5,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::LandingCraft,
                count: 4,
                spacing: 2.5,
                lead_in: 4.7,
            },
            Group {
                kind: EnemyKind::Destroyer,
                count: 2,
                spacing: 4.0,
                lead_in: 11.2,
            },
        ],
    },
    Wave {
        name: "Jagdgruppe",
        groups: &[
            Group {
                kind: EnemyKind::Corvette,
                count: 6,
                spacing: 1.6,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::Destroyer,
                count: 3,
                spacing: 4.0,
                lead_in: 6.8,
            },
        ],
    },
    Wave {
        name: "Großangriff",
        groups: &[
            Group {
                kind: EnemyKind::MotorTorpedoBoat,
                count: 20,
                spacing: 0.5,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::Corvette,
                count: 8,
                spacing: 1.6,
                lead_in: 7.7,
            },
            Group {
                kind: EnemyKind::LandingCraft,
                count: 4,
                spacing: 2.5,
                lead_in: 16.4,
            },
        ],
    },
    Wave {
        name: "Vorhut",
        groups: &[
            Group {
                kind: EnemyKind::MotorTorpedoBoat,
                count: 8,
                spacing: 0.5,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::Destroyer,
                count: 4,
                spacing: 4.0,
                lead_in: 4.1,
            },
            Group {
                kind: EnemyKind::LandingCraft,
                count: 5,
                spacing: 2.5,
                lead_in: 13.3,
            },
        ],
    },
    Wave {
        name: "Der Monitor",
        groups: &[
            Group {
                kind: EnemyKind::MotorTorpedoBoat,
                count: 16,
                spacing: 0.5,
                lead_in: 0.0,
            },
            Group {
                kind: EnemyKind::Destroyer,
                count: 4,
                spacing: 4.0,
                lead_in: 6.5,
            },
            Group {
                kind: EnemyKind::Monitor,
                count: 2,
                spacing: 12.0,
                lead_in: 15.7,
            },
        ],
    },
];
