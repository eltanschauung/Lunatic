// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;

fn strength_bonus(oni: bool, yuuka: bool, fist: bool, strength: Option<i32>) -> f32 {
    let desired: f32 = if oni && fist { 6.0 } else if oni || yuuka { 3.0 } else { 0.0 };
    let vanilla = strength.map_or(0.0, |level| 3.0 * (level.max(0) as f32 + 1.0));
    (desired - vanilla).max(0.0)
}
impl Game<'_> {
    pub(super) fn lunatic_melee_bonus(&self) -> f32 {
        if !self.native_on("lunatic") { return 0.0; }
        strength_bonus(
            self.potions.effects.get("lunatic:oni").is_some(),
            self.lunatic_has_race("yuuka"),
            self.held().is_none(),
            self.potions.effects.amplifier("strength"),
        )
    }
    pub(super) fn lunatic_punch_effects(&mut self, index: usize) {
        if self.held().is_some() || !self.native_on("lunatic")
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
            spread: [0.0; 3], speed: 0.0, count: 1, randomization: 0,
        });
    }
}
#[cfg(test)]
mod tests {
    use super::strength_bonus;
    #[test] fn oni_distinguishes_fists_and_items() {
        assert_eq!(strength_bonus(true, false, true, None), 6.0);
        assert_eq!(strength_bonus(true, false, false, None), 3.0);
    }
    #[test] fn yuuka_is_permanent_and_never_stacks_with_oni_or_strength() {
        for fist in [false, true] {
            assert_eq!(strength_bonus(false, true, fist, None), 3.0);
            assert_eq!(strength_bonus(true, true, fist, None), if fist {6.0} else {3.0});
        }
        assert_eq!(strength_bonus(true, true, true, Some(0)), 3.0);
        assert_eq!(strength_bonus(true, true, true, Some(1)), 0.0);
        assert_eq!(strength_bonus(false, true, false, Some(2)), 0.0);
        assert_eq!(strength_bonus(false, false, true, None), 0.0);
    }
}
