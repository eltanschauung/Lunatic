// SPDX-License-Identifier: GPL-3.0-or-later
// Luanti's one airborne boost and cancellable float, in Minecraft blocks/tick.
use super::PlayerMotion;

#[derive(Clone, Debug)]
pub struct Fairy {
    pub enabled: bool,
    used: bool,
    previous_jump: bool,
    ascending: bool,
    pub slowfall: bool,
    potion_slowfall: bool,
    previous_velocity: f64,
    size: f32,
    pub sound: Option<&'static str>,
    pub kit_checked: Option<String>,
}
impl Default for Fairy {
    fn default() -> Self {
        Self {
            enabled: false,
            used: false,
            previous_jump: false,
            ascending: false,
            slowfall: false,
            potion_slowfall: false,
            previous_velocity: 0.0,
            size: 1.0,
            sound: None,
            kit_checked: None,
        }
    }
}
#[derive(Clone, Copy)]
struct Input {
    jump: bool,
    grounded: bool,
    blocked: bool,
    velocity: f64,
}
impl Fairy {
    fn step(&mut self, input: Input) -> bool {
        let pressed = input.jump && !self.previous_jump;
        self.previous_jump = input.jump;
        if !self.enabled || input.grounded || input.blocked {
            self.used = false;
            self.ascending = false;
            self.slowfall = false;
            self.previous_velocity = input.velocity;
            return false;
        }
        let boost = pressed && !self.used;
        if boost {
            self.used = true;
            self.slowfall = false;
            self.ascending = true;
            self.sound = Some("entity.fairy.jump");
        } else if pressed && self.slowfall {
            self.slowfall = false;
            self.ascending = false;
            self.sound = Some("entity.fairy.cancel");
        } else if self.ascending && input.velocity <= self.previous_velocity {
            self.ascending = false;
            self.slowfall = true;
        }
        self.previous_velocity = input.velocity + if boost { 12.0 / 20.0 } else { 0.0 };
        boost
    }
}
impl PlayerMotion {
    pub fn lunatic_character_body(&mut self, size: f32, fairy: bool, potion_slowfall: bool) {
        let size = if size.is_finite() {
            size.clamp(0.7, 1.2)
        } else {
            1.0
        };
        let state = self.__horizon_mixins_lunatic_fairy();
        let changed = state.size != size;
        let has = self
            .attributes
            .key("minecraft:scale")
            .and_then(|key| {
                self.attributes
                    .has_modifier(Some(key), "lunatic:character_size")
                    .ok()
            })
            .unwrap_or(false);
        if changed || has != (size != 1.0) {
            let _ = self.set_modifier(
                "minecraft:scale",
                "lunatic:character_size",
                f64::from(size) - 1.0,
                "add_multiplied_total",
                size != 1.0,
            );
            let _ = self.refresh_dimensions();
        }
        let state = self.__horizon_mixins_lunatic_fairy_mut();
        state.size = size;
        state.enabled = fairy;
        state.potion_slowfall = potion_slowfall;
        if !fairy {
            state.used = false;
            state.slowfall = false;
            state.ascending = false;
        }
        self.status.slow_falling =
            potion_slowfall || self.__horizon_mixins_lunatic_fairy().slowfall;
    }
    pub fn lunatic_fairy_sync(&mut self, server: &Self) {
        let enabled = server.__horizon_mixins_lunatic_fairy().enabled;
        let potion = server.__horizon_mixins_lunatic_fairy().potion_slowfall;
        let state = self.__horizon_mixins_lunatic_fairy_mut();
        state.enabled = enabled;
        state.potion_slowfall = potion;
        if !enabled {
            state.used = false;
            state.slowfall = false;
            state.ascending = false;
        }
    }
    pub fn lunatic_fairy_step(&mut self, jump: bool, grounded: bool) {
        let input = Input {
            jump,
            grounded,
            velocity: self.velocity.y,
            blocked: self.health.dead_or_dying()
                || self.flying
                || self.spectator
                || self.is_passenger
                || self.sleeping
                || self.fall_flying
                || self.fluids.was_touching_water
                || self.is_in_lava(),
        };
        let state = self.__horizon_mixins_lunatic_fairy_mut();
        let enabled = state.enabled;
        let boost = state.step(input);
        // An ordinary body keeps every vanilla status flag, including flags
        // supplied directly by the potion/movement pipeline without a Game.
        if !enabled {
            return;
        }
        let slowfall = state.potion_slowfall || state.slowfall;
        if boost {
            self.velocity.y += 12.0 / 20.0;
            self.fall_distance = 0.0;
        }
        // Exactly the status consumed by LivingEntity.calculateEffectiveGravity and
        // Player.checkFallDamage for the vanilla Slow Falling potion; no custom gravity.
        self.status.slow_falling = slowfall;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn input(jump: bool, grounded: bool, velocity: f64) -> Input {
        Input {
            jump,
            grounded,
            blocked: false,
            velocity,
        }
    }
    #[test]
    fn fairy_boost_is_once_per_airtime_and_a_second_press_cancels_float() {
        let mut state = Fairy {
            enabled: true,
            ..Fairy::default()
        };
        assert!(!state.step(input(true, true, 0.0)));
        assert!(!state.step(input(true, false, 0.3)));
        assert!(!state.step(input(false, false, 0.2)));
        assert!(state.step(input(true, false, 0.1)));
        assert!(!state.step(input(true, false, 0.4)));
        assert!(state.slowfall);
        state.step(input(false, false, -0.01));
        assert!(!state.step(input(true, false, -0.01)));
        assert!(!state.slowfall);
        state.step(input(false, false, -0.1));
        assert!(!state.step(input(true, false, -0.1)));
        state.step(input(false, true, 0.0));
        state.step(input(true, true, 0.0));
        state.step(input(false, false, 0.2));
        assert!(state.step(input(true, false, 0.1)));
    }
    #[test]
    fn body_scale_uses_all_pose_dimensions_and_float_preserves_the_potion_source() {
        use crate::{assets::Assets, motion::MotionResources, pose::Pose};
        use horizon_core::geometry::Vec3;
        let root = std::path::PathBuf::from(
            std::env::var_os("HORIZON_TEST_RESOURCES")
                .expect("race regression requires prepared resources"),
        );
        let assets = Assets::load(&root).unwrap();
        let resources = MotionResources::load(&root, &assets).unwrap();
        let mut p = PlayerMotion::new(&resources, Vec3::ZERO).unwrap();
        p.lunatic_character_body(0.7, true, false);
        assert!((p.scale - 0.7).abs() < 1e-6);
        for pose in [
            Pose::Standing,
            Pose::Crouching,
            Pose::Swimming,
            Pose::FallFlying,
        ] {
            p.set_pose(pose).unwrap();
            let expected = pose.dimensions(0.7);
            assert_eq!(p.dimensions.width, expected.width);
            assert_eq!(p.dimensions.height, expected.height);
            assert_eq!(p.dimensions.eye_height, expected.eye_height);
        }
        p.set_pose(Pose::Standing).unwrap();
        let width = p.bounds().max_position().x - p.bounds().min_position().x;
        assert!((width - f64::from(0.6f32 * 0.7)).abs() < 1e-6);
        p.lunatic_fairy_step(false, true);
        p.lunatic_fairy_step(true, false);
        p.velocity.y -= 0.08;
        p.lunatic_fairy_step(false, false);
        assert!(p.status.slow_falling);
        p.lunatic_character_body(0.7, true, true);
        p.lunatic_fairy_step(true, false);
        assert!(!p.__horizon_mixins_lunatic_fairy().slowfall);
        assert!(
            p.status.slow_falling,
            "a potion survives race float cancellation"
        );
        p.lunatic_character_body(1.0, false, false);
        assert!(!p.status.slow_falling);
        assert_eq!(p.scale, 1.0);
        assert_eq!(p.dimensions.eye_height, 1.62);
        p.lunatic_character_body(4.0, false, false);
        assert!((p.scale - 1.2).abs() < 1e-6);
    }
    #[test]
    fn nonfairies_and_flight_water_riding_do_not_get_a_boost() {
        let mut state = Fairy::default();
        assert!(!state.step(input(true, false, -0.1)));
        state.enabled = true;
        state.step(input(false, false, 0.0));
        assert!(!state.step(Input {
            blocked: true,
            ..input(true, false, 0.0)
        }));
        assert!(!state.slowfall);
    }
}
