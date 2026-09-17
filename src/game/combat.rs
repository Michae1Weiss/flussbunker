//! Health, armour, damage and death for everything that can sink.
//! Sole writer of `Health`, `LastHitBy`; `Damage` messages in, `Sunk` events out.

use bevy::prelude::*;

use crate::game_plugin::Sim;

pub fn plugin(app: &mut App) {
    app.add_message::<Damage>().add_systems(
        FixedUpdate,
        (
            apply_damage.in_set(Sim::Damage),
            sink_the_dead.in_set(Sim::Death),
        ),
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

/// Flat reduction per shell hit; absent = unarmoured.
#[derive(Component, Debug, Default)]
pub struct Armor(pub f32);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DamageKind {
    /// Reduced by armour.
    Shell,
    /// Ignores armour.
    Torpedo,
    /// Ignores armour; collisions with the U-boat.
    Ram,
}

#[derive(Message, Debug)]
pub struct Damage {
    pub target: Entity,
    pub amount: f32,
    pub kind: DamageKind,
}

/// Kind of the most recent hit; inserted by `apply_damage`.
#[derive(Component, Debug)]
pub struct LastHitBy(pub DamageKind);

/// Triggered before the despawn, so observers still read the entity's components.
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
        let Ok((mut health, armor)) = targets.get_mut(hit.target) else {
            continue;
        };

        let effective = match hit.kind {
            // 10 % floor: armour never blocks a shell entirely.
            DamageKind::Shell => {
                let armor = armor.map_or(0.0, |a| a.0);
                (hit.amount - armor).max(hit.amount * 0.1)
            }
            DamageKind::Torpedo | DamageKind::Ram => hit.amount,
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
        // Commands run in queue order: observers see a live entity.
        commands.trigger(Sunk { entity });
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_message::<Damage>();
        app.add_systems(Update, apply_damage);
        app
    }

    fn hit(app: &mut App, target: Entity, amount: f32, kind: DamageKind) {
        app.world_mut()
            .resource_mut::<Messages<Damage>>()
            .write(Damage {
                target,
                amount,
                kind,
            });
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

        assert_eq!(
            app.world().entity(boat).get::<Health>().unwrap().current,
            80.0
        );
    }

    #[test]
    fn armor_never_blocks_everything() {
        let mut app = test_app();
        let boat = app
            .world_mut()
            .spawn((Health::new(100.0), Armor(999.0)))
            .id();

        hit(&mut app, boat, 10.0, DamageKind::Shell);

        assert_eq!(
            app.world().entity(boat).get::<Health>().unwrap().current,
            99.0
        );
    }
}
