//! What the player sets in the settings menu: music and sound volume in steps, onboarding
//! hints on or off. Lives for the session, not stored. Sole writer: `screens::settings`.

use bevy::prelude::*;

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Settings {
    pub music: u8,
    pub sound: u8,
    pub hints: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            music: 6,
            sound: 8,
            hints: true,
        }
    }
}

impl Settings {
    pub const STEPS: u8 = 10;

    /// Linear gain for a volume step. Squared: loudness is heard logarithmically, so equal
    /// steps then sound equal.
    pub fn gain(step: u8) -> f32 {
        let x = step.min(Self::STEPS) as f32 / Self::STEPS as f32;
        x * x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gain_runs_from_silence_to_full_and_never_falls() {
        assert_eq!(Settings::gain(0), 0.0);
        assert_eq!(Settings::gain(Settings::STEPS), 1.0);
        assert_eq!(Settings::gain(Settings::STEPS + 3), 1.0);
        for step in 0..Settings::STEPS {
            assert!(Settings::gain(step) < Settings::gain(step + 1));
        }
    }
}
