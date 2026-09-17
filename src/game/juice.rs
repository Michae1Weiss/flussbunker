//! Feedback that changes no game state: enemies flash white when hit, the camera shakes
//! on torpedo hits and blasts. Sole writer of `HitFlash`, `Shake`, `Juice` and of the
//! camera's `Transform`. Rules: docs/design/destruction.md#presentation-post-greybox

use bevy::prelude::*;

use crate::balance::{FLASH_SECONDS, SHAKE_DECAY, SHAKE_MAX_OFFSET};
use crate::game::combat::{Damage, DamageKind, Weapon};
use crate::game::enemy::Enemy;
use crate::game_plugin::Frame;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.init_resource::<Juice>()
        .init_resource::<Shake>()
        .add_systems(OnEnter(Screen::Gameplay), reset)
        .add_systems(
            Update,
            (react_to_damage, end_flashes, shake_camera)
                .chain()
                .in_set(Frame::Present),
        );
}

/// Player settings for feedback that some people cannot stand.
#[derive(Resource)]
pub struct Juice {
    pub shake: bool,
}

impl Default for Juice {
    fn default() -> Self {
        Self { shake: true }
    }
}

/// Trauma in 0..=1; the offset is trauma squared so small hits barely move the view.
#[derive(Resource, Default)]
pub struct Shake {
    pub trauma: f32,
}

#[derive(Component)]
pub struct HitFlash {
    timer: Timer,
    base: Color,
}

fn reset(mut shake: ResMut<Shake>) {
    shake.trauma = 0.0;
}

fn react_to_damage(
    mut hits: MessageReader<Damage>,
    mut shake: ResMut<Shake>,
    mut commands: Commands,
    mut enemies: Query<(&mut Sprite, Option<&HitFlash>), With<Enemy>>,
) {
    for hit in hits.read() {
        shake.trauma = (shake.trauma
            + match (hit.weapon, hit.kind) {
                (Weapon::Torpedo, _) => 0.45,
                (_, DamageKind::Blast) => 0.25,
                _ => 0.0,
            })
        .min(1.0);

        let Ok((mut sprite, flashing)) = enemies.get_mut(hit.target) else {
            continue;
        };
        if flashing.is_some() {
            continue;
        }
        let base = sprite.color;
        sprite.color = Color::WHITE;
        commands.entity(hit.target).insert(HitFlash {
            timer: Timer::from_seconds(FLASH_SECONDS, TimerMode::Once),
            base,
        });
    }
}

fn end_flashes(
    time: Res<Time>,
    mut commands: Commands,
    mut flashing: Query<(Entity, &mut Sprite, &mut HitFlash)>,
) {
    for (entity, mut sprite, mut flash) in &mut flashing {
        if flash.timer.tick(time.delta()).just_finished() {
            sprite.color = flash.base;
            commands.entity(entity).remove::<HitFlash>();
        }
    }
}

fn shake_camera(
    time: Res<Time>,
    juice: Res<Juice>,
    mut shake: ResMut<Shake>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
) {
    shake.trauma = (shake.trauma - SHAKE_DECAY * time.delta_secs()).max(0.0);
    let amount = if juice.shake {
        shake.trauma * shake.trauma * SHAKE_MAX_OFFSET
    } else {
        0.0
    };
    // Cheap noise: two incommensurate sines of elapsed time.
    let t = time.elapsed_secs() * 60.0;
    let offset = Vec2::new((t * 1.3).sin(), (t * 1.7 + 1.0).sin()) * amount;
    camera.translation = offset.extend(camera.translation.z);
}
