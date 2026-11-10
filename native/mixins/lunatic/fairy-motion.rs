// SPDX-License-Identifier: GPL-3.0-or-later
// Luanti's one airborne boost and cancellable float, in Minecraft blocks/tick.
use super::PlayerMotion;

#[derive(Clone, Debug)]
pub struct Fairy {
    pub enabled: bool,
    pub crow_tengu: bool,
    pub crow_armor_blocked: bool,
    pub oni_ending: bool,
    used: bool,
    previous_jump: bool,
    ascending: bool,
    pub slowfall: bool,
    pub parasol: bool,
    pub parasol_sent: Option<bool>,
    pub parasol_revision: u64,
    potion_slowfall: bool,
    previous_velocity: f64,
    size: f32,
    pub sound: Option<&'static str>,
    pub character_health: f32,
    pub kit_checked: Option<String>,
}
impl Default for Fairy {
    fn default() -> Self {
        Self {
            enabled: false,
            crow_tengu: false,
            crow_armor_blocked: false,
            oni_ending: false,
            used: false,
            previous_jump: false,
            ascending: false,
            slowfall: false,
            parasol: false,
            parasol_sent: None,
            parasol_revision: 0,
            potion_slowfall: false,
            previous_velocity: 0.0,
            size: 1.0,
            sound: None,
            character_health: 20.0,
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
        if pressed && self.parasol {
            if input.grounded { self.used = false; }
            self.parasol = false;
            self.sound = Some("entity.fairy.cancel");
            self.previous_velocity = input.velocity;
            return false;
        }
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
    /// Reuse the native attribute instance's dirty/computed cache. No equipment
    /// traversal, extra packets, or separately invalidated armor-value cache.
    pub fn lunatic_crow_glide(&mut self) {
        let crow = self.__horizon_mixins_lunatic_fairy().crow_tengu;
        let blocked = crow && self.attribute("minecraft:armor").map_or(true, |v| v > 11.0);
        self.__horizon_mixins_lunatic_fairy_mut().crow_armor_blocked = blocked;
        if crow {
            self.glider = !blocked;
            if blocked {
                self.fall_flying = false;
            }
        }
    }
    pub fn lunatic_character_body(
        &mut self,
        size: f32,
        fairy: bool,
        potion_slowfall: bool,
        crow_tengu: bool,
    ) {
        let size = if size.is_finite() {
            size.clamp(0.7, 1.2)
        } else {
            1.0
        };
        let state = self.__horizon_mixins_lunatic_fairy();
        let changed = state.size != size;
        let crow_changed = state.crow_tengu != crow_tengu;
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
        state.crow_tengu = crow_tengu;
        state.potion_slowfall = potion_slowfall;
        if !fairy {
            state.used = false;
            state.slowfall = false;
            state.ascending = false;
        }
        self.status.slow_falling =
            potion_slowfall || self.__horizon_mixins_lunatic_fairy().slowfall
                || self.__horizon_mixins_lunatic_fairy().parasol;
        if crow_changed {
            self.lunatic_crow_glide();
        }
    }
    pub fn lunatic_fairy_sync(&mut self, server: &Self) {
        let enabled = server.__horizon_mixins_lunatic_fairy().enabled;
        let crow_tengu = server.__horizon_mixins_lunatic_fairy().crow_tengu;
        let potion = server.__horizon_mixins_lunatic_fairy().potion_slowfall;
        let parasol = server.__horizon_mixins_lunatic_fairy().parasol;
        let revision = server.__horizon_mixins_lunatic_fairy().parasol_revision;
        let state = self.__horizon_mixins_lunatic_fairy_mut();
        state.enabled = enabled;
        state.crow_tengu = crow_tengu;
        state.potion_slowfall = potion;
        // Replay derives Space cancellation from the recorded input. Only a
        // new item toggle/death reset overrides it, not a later input's result.
        if state.parasol_revision != revision {
            state.parasol = parasol;
            state.parasol_revision = revision;
        }
        if !enabled {
            state.used = false;
            state.slowfall = false;
            state.ascending = false;
        }
    }
    pub fn lunatic_parasol_set(&mut self, active: bool) {
        let state = self.__horizon_mixins_lunatic_fairy_mut();
        if state.parasol != active {
            state.parasol = active;
            state.parasol_revision = state.parasol_revision.wrapping_add(1);
        }
        if active {
            state.slowfall = false;
            state.ascending = false;
        }
        self.status.slow_falling = state.potion_slowfall || state.slowfall || state.parasol;
    }
    pub fn lunatic_fairy_step(&mut self, jump: bool, grounded: bool) {
        if self.health.dead_or_dying() || self.spectator {
            self.lunatic_parasol_set(false);
        }
        self.lunatic_crow_glide();
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
        let controlled = state.enabled || state.parasol;
        let boost = state.step(input);
        // An ordinary body keeps every vanilla status flag, including flags
        // supplied directly by the potion/movement pipeline without a Game.
        if !controlled {
            return;
        }
        let slowfall = state.potion_slowfall || state.slowfall || state.parasol;
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
    fn parasol_works_for_nonfairies_and_space_cancels_without_boost() {
        for fairy in [false, true] {
            let mut state = Fairy { enabled: fairy, parasol: true, ..Fairy::default() };
            assert!(!state.step(input(false, true, 0.0)));
            assert!(state.parasol, "opening on the ground remains armed");
            assert!(!state.step(input(false, false, -0.1)));
            assert!(state.parasol);
            assert!(!state.step(input(true, false, -0.1)));
            assert!(!state.parasol);
            assert_eq!(state.sound, Some("entity.fairy.cancel"));
            assert!(!state.step(input(true, false, -0.2)), "holding Space cannot boost");
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
        p.lunatic_character_body(0.7, true, false, false);
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
        p.lunatic_character_body(0.7, true, true, false);
        p.lunatic_fairy_step(true, false);
        assert!(!p.__horizon_mixins_lunatic_fairy().slowfall);
        assert!(
            p.status.slow_falling,
            "a potion survives race float cancellation"
        );
        p.lunatic_character_body(1.0, false, false, false);
        assert!(!p.status.slow_falling);
        assert_eq!(p.scale, 1.0);
        assert_eq!(p.dimensions.eye_height, 1.62);
        p.lunatic_character_body(4.0, false, false, false);
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
