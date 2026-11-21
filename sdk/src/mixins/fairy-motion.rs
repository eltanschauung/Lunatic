// SPDX-License-Identifier: GPL-3.0-or-later
// Luanti's one airborne boost and cancellable float, in Minecraft blocks/tick.
use horizon_client_lib::motion::{PlayerMotion, ReplayAttachment};
use ::mod_lunatic::fairy_state::{Fairy, Input};

static FAIRY: ReplayAttachment<Fairy> = ReplayAttachment::new("lunatic", "fairy", Fairy::default);
static EMPTY: std::sync::LazyLock<Fairy> = std::sync::LazyLock::new(Fairy::default);

impl PlayerMotionLunaticFairyMotionExt for PlayerMotion {
     fn lunatic_fairy(&self) -> &Fairy {
        self.replay_attachment(&FAIRY).unwrap_or(&EMPTY)
    }
     fn lunatic_edit_fairy<R>(&mut self, edit: impl FnOnce(&mut Fairy) -> R) -> R {
        self.edit_replay_attachment(&FAIRY, edit)
            .expect("Lunatic owns the fairy replay attachment key")
    }

    /// Reuse the native attribute instance's dirty/computed cache. No equipment
    /// traversal, extra packets, or separately invalidated armor-value cache.
     fn lunatic_racial_glide(&mut self) {
        let crow = self.lunatic_fairy().racial_glider;
        let blocked = crow && self.attribute("minecraft:armor").map_or(true, |v| v > 11.0);
        self.lunatic_edit_fairy(|state| state.racial_glide_armor_blocked = blocked);
        if crow {
            self.glider = !blocked;
            if blocked {
                self.fall_flying = false;
            }
        }
    }
     fn lunatic_character_body(
        &mut self,
        size: f32,
        fairy: bool,
        potion_slowfall: bool,
        racial_glider: bool,
    ) {
        let size = if size.is_finite() {
            size.clamp(0.7, 1.2)
        } else {
            1.0
        };
        let state = self.lunatic_fairy();
        let changed = state.size != size;
        let crow_changed = state.racial_glider != racial_glider;
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
        self.status.slow_falling = self.lunatic_edit_fairy(|state| {
            state.size = size;
            state.enabled = fairy;
            state.racial_glider = racial_glider;
            state.potion_slowfall = potion_slowfall;
            if !fairy {
                state.used = false;
                state.slowfall = false;
                state.ascending = false;
            }
            potion_slowfall || state.slowfall || state.parasol
        });
        if crow_changed {
            self.lunatic_racial_glide();
        }
    }
     fn lunatic_fairy_sync(&mut self, server: &Self) {
        let server = server.lunatic_fairy();
        self.lunatic_edit_fairy(|state| {
            state.enabled = server.enabled;
            state.racial_glider = server.racial_glider;
            state.potion_slowfall = server.potion_slowfall;
            // Replay derives Space cancellation from the recorded input. Only a
            // new item toggle or death reset overrides that input's result.
            if state.parasol_revision != server.parasol_revision {
                state.parasol = server.parasol;
                state.parasol_revision = server.parasol_revision;
            }
            if !server.enabled {
                state.used = false;
                state.slowfall = false;
                state.ascending = false;
            }
        });
    }

     fn lunatic_parasol_set(&mut self, active: bool) {
        self.status.slow_falling = self.lunatic_edit_fairy(|state| {
            if state.parasol != active {
                state.parasol = active;
                state.parasol_revision = state.parasol_revision.wrapping_add(1);
            }
            if active {
                state.slowfall = false;
                state.ascending = false;
            }
            state.potion_slowfall || state.slowfall || state.parasol
        });
    }

     fn lunatic_fairy_step(&mut self, jump: bool, grounded: bool) {
        if self.health.dead_or_dying() || self.spectator {
            self.lunatic_parasol_set(false);
        }
        self.lunatic_racial_glide();
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
        let (controlled, boost, slowfall) = self.lunatic_edit_fairy(|state| {
            let controlled = state.enabled || state.parasol;
            let boost = state.step(input);
            (controlled, boost, state.potion_slowfall || state.slowfall || state.parasol)
        });
        // An ordinary body keeps every vanilla status flag, including flags
        // supplied directly by the potion/movement pipeline without a Game.
        if !controlled {
            return;
        }
        if boost {
            self.velocity.y += 12.0 / 20.0;
            self.fall_distance = 0.0;
        }
        // Exactly the status consumed by LivingEntity.calculateEffectiveGravity and
        // Player.checkFallDamage for the vanilla Slow Falling potion; no custom gravity.
        self.status.slow_falling = slowfall;
    }
}

pub trait PlayerMotionLunaticFairyMotionExt {
    fn lunatic_fairy(&self) -> &Fairy;
    fn lunatic_edit_fairy<R>(&mut self, edit: impl FnOnce(&mut Fairy) -> R) -> R;
    fn lunatic_racial_glide(&mut self);
    fn lunatic_character_body(
        &mut self,
        size: f32,
        fairy: bool,
        potion_slowfall: bool,
        racial_glider: bool,
    );
    fn lunatic_fairy_sync(&mut self, server: &Self);
    fn lunatic_parasol_set(&mut self, active: bool);
    fn lunatic_fairy_step(&mut self, jump: bool, grounded: bool);
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
        assert!(!p.lunatic_fairy().slowfall);
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
