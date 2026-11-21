// SPDX-License-Identifier: GPL-3.0-or-later
use crate::game::lunatic_orbs::GameLunaticRuntimeExt as _;
use crate::game::lunatic_races::GameLunaticRacesExt as _;
use super::Game;

fn strength_bonus(oni: bool, yuuka: bool, fist: bool, strength: Option<i32>) -> f32 {
    if !oni {
        return if yuuka && fist { 2.0 } else { 0.0 };
    }
    let desired: f32 = if fist { 6.0 } else { 3.0 };
    let vanilla = strength.map_or(0.0, |level| 3.0 * (level.max(0) as f32 + 1.0));
    (desired - vanilla).max(0.0)
}
impl GameLunaticMeleeExt for Game<'_> {
     fn lunatic_melee_bonus(&self) -> f32 {
        if !self.on("lunatic") { return 0.0; }
        strength_bonus(
            self.potions.effects.get("lunatic:oni").is_some(),
            self.lunatic_has_race("yuuka"),
            self.held().is_none(),
            self.potions.effects.amplifier("strength"),
        )
    }
     fn lunatic_punch_effects(&mut self, index: usize) {
        if self.held().is_some() || !self.on("lunatic")
            || !(self.potions.effects.get("lunatic:oni").is_some() || self.lunatic_has_race("yuuka")) {
            return;
        }
        let Some((origin, height)) = self.mobs.get(index)
            .filter(|m| m.kind.id != "minecraft:player")
            .map(|m| (m.position, m.height())) else { return; };
        self.lunatic_sound("entity.explosion_old", origin, 0.5);
        // Visual impact only; normal particle dispatch reaches nearby clients.
        let options = horizon_client::particles::ParticleOptions::simple(
            horizon_client::particles::ParticleType::Explosion,
        );
        let at = [origin.x, origin.y + f64::from(height) * 0.5, origin.z];
        self.particles.add(&options, at, [0.0; 3]);
        self.lan_send_particles(super::lan_particles::QueuedParticle {
            dimension: self.dim.id(), options, state: None, at,
            spread: [0.0; 3], speed: [0.0; 3], count: 1, randomization: 0,
        });
    }
}

pub trait GameLunaticMeleeExt {
    fn lunatic_melee_bonus(&self) -> f32;
    fn lunatic_punch_effects(&mut self, index: usize);
}
#[cfg(test)]
mod tests {
    use super::strength_bonus;
    #[test] fn oni_distinguishes_fists_and_items() {
        assert_eq!(strength_bonus(true, false, true, None), 6.0);
        assert_eq!(strength_bonus(true, false, false, None), 3.0);
    }
    #[test] fn yuukas_fist_bonus_does_not_replace_vanilla_strength() {
        for fist in [false, true] {
            assert_eq!(strength_bonus(false, true, fist, None), if fist { 2.0 } else { 0.0 });
            assert_eq!(strength_bonus(true, true, fist, None), if fist {6.0} else {3.0});
        }
        assert_eq!(strength_bonus(true, true, true, Some(0)), 3.0);
        assert_eq!(strength_bonus(true, true, true, Some(1)), 0.0);
        assert_eq!(strength_bonus(false, true, true, Some(0)), 2.0);
        assert_eq!(strength_bonus(false, true, false, Some(2)), 0.0);
        assert_eq!(strength_bonus(false, false, true, None), 0.0);
    }
}
