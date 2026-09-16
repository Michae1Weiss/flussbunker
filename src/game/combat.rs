//! The shared combat contract: `Health`, `Armor`, `Damage`, `Sunk`.
//!
//! Towers, u-boats and destroyers all want to hurt something. Instead of three
//! "attack" systems that each reach into health and each re-implement armour,
//! everyone writes one `Damage` message and this module owns the arithmetic.
//!
//! This is the one module several features depend on, and that is fine: the
//! dependencies all point *inward*, to a module that depends on nobody. A
//! shared module becomes a problem only when it starts depending back.
//!
//! Note the two communication styles side by side, chosen for opposite reasons:
//!
//! * `Damage` is a **Message** — many writers, high frequency, order matters
//!   (all damage must land before we check who died), and no writer cares who
//!   consumes it.
//! * `Sunk` is an **EntityEvent** — one specific entity, and the reactions
//!   (bounty, pen slot, particles) want to run right now, on that entity,
//!   before it is despawned.

use bevy::prelude::*;

use crate::game_plugin::Sim;

pub(super) fn plugin(app: &mut App) {
    app.add_message::<Damage>().add_systems(
        FixedUpdate,
        (apply_damage.in_set(Sim::Damage), sink_the_dead.in_set(Sim::Death)),
    );
}

#[derive(Component, Debug)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn fraction(&self) -> f32 {
        (self.current / self.max).clamp(0.0, 1.0)
    }
}

/// Flat reduction per shell hit. Absent = unarmoured.
#[derive(Component, Debug, Default)]
pub struct Armor(pub f32);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DamageKind {
    /// Reduced by armour.
    Shell,
    /// Ignores armour. Only u-boats deal it.
    Torpedo,
}

#[derive(Message, Debug)]
pub struct Damage {
    pub target: Entity,
    pub amount: f32,
    pub kind: DamageKind,
}

/// What last hurt this entity. One tiny component, added by the damage system,
/// so that "who gets the kill credit" is answerable without threading an
/// attacker `Entity` through every projectile and every message.
///
/// It records the *kind*, not the shooter — because the only question the game
/// actually asks is "was this a torpedo kill", for u-boat veterancy. Storing
/// the attacker would be more general and less useful: general-purpose data
/// nobody queries is just a field that can go stale.
#[derive(Component, Debug)]
pub struct LastHitBy(pub DamageKind);

/// Fired on an entity the instant its health hits zero, *before* it despawns —
/// so observers can still read its components.
#[derive(EntityEvent, Debug)]
pub struct Sunk {
    #[event_target]
    pub entity: Entity,
}

fn apply_damage(
    mut commands: Commands,
    mut incoming: MessageReader<Damage>,
    mut targets: Query<(&mut Health, Option<&Armor>)>,
) {
    for hit in incoming.read() {
        // The target may already be despawned this tick — two towers can fire
        // at the same boat. `let Ok(..) else` makes that a non-event rather
        // than an error to handle.
        let Ok((mut health, armor)) = targets.get_mut(hit.target) else {
            continue;
        };

        let effective = match hit.kind {
            // 10% always gets through: a floor keeps armour from creating a
            // frustrating "literally nothing happens" state, while still
            // making the wrong tool obviously wrong.
            DamageKind::Shell => {
                let armor = armor.map_or(0.0, |a| a.0);
                (hit.amount - armor).max(hit.amount * 0.1)
            }
            DamageKind::Torpedo => hit.amount,
        };

        health.current -= effective;
        commands.entity(hit.target).insert(LastHitBy(hit.kind));
    }
}

fn sink_the_dead(mut commands: Commands, dying: Query<(Entity, &Health), Changed<Health>>) {
    for (entity, health) in &dying {
        if health.current > 0.0 {
            continue;
        }
        // Order matters and is guaranteed: commands run in queue order, so the
        // trigger is processed (and its observers run to completion) before the
        // despawn. Observers therefore still see a live entity.
        commands.trigger(Sunk { entity });
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Headless `App`, one tick, assert on the world. No window, no renderer,
    /// runs in milliseconds — this is the shape of every worthwhile Bevy test.
    fn test_app() -> App {
        let mut app = App::new();
        app.add_message::<Damage>();
        app.add_systems(Update, apply_damage);
        app
    }

    fn hit(app: &mut App, target: Entity, amount: f32, kind: DamageKind) {
        app.world_mut()
            .resource_mut::<Messages<Damage>>()
            .write(Damage { target, amount, kind });
        app.update();
    }

    #[test]
    fn armor_reduces_shells() {
        let mut app = test_app();
        let boat = app.world_mut().spawn((Health::new(100.0), Armor(7.0))).id();

        hit(&mut app, boat, 20.0, DamageKind::Shell);

        let health = app.world().entity(boat).get::<Health>().unwrap();
        assert_eq!(health.current, 87.0);
    }

    #[test]
    fn torpedoes_ignore_armor() {
        let mut app = test_app();
        let boat = app.world_mut().spawn((Health::new(100.0), Armor(7.0))).id();

        hit(&mut app, boat, 20.0, DamageKind::Torpedo);

        assert_eq!(app.world().entity(boat).get::<Health>().unwrap().current, 80.0);
    }

    /// The design promise "guns are bad against armour, but never useless".
    /// Encoding it as a test means a later balance pass cannot break it silently.
    #[test]
    fn armor_never_blocks_everything() {
        let mut app = test_app();
        let boat = app.world_mut().spawn((Health::new(100.0), Armor(999.0))).id();

        hit(&mut app, boat, 10.0, DamageKind::Shell);

        assert_eq!(app.world().entity(boat).get::<Health>().unwrap().current, 99.0);
    }
}
