// SPDX-License-Identifier: GPL-3.0-or-later
// Luanti particlespawners: eight independent sprites, not eight animation frames.
use super::{Particle, ParticleEngine, ParticleType, Sprite};
use horizon_core::random::RandomSource;

const PREFIX: &str = "lunatic:particle/yin_yang_orb/";
fn ours(p: &Particle) -> bool {
    matches!(&p.sprite, Sprite::Named(name) if name.starts_with(PREFIX))
}
pub fn tick(p: &mut Particle) -> bool {
    if !ours(p) {
        return false;
    }
    if p.age < 0 {
        p.age += 1;
        p.no_render = p.age < 0;
        return true;
    }
    p.previous = p.position;
    p.age += 1;
    if p.age >= p.lifetime {
        p.removed = true;
        return true;
    }
    for i in 0..3 {
        p.position[i] += p.velocity[i];
    }
    true
}
pub fn light(p: &Particle, sky: u32, block: u32) -> Option<(u32, u32)> {
    ours(p).then_some((sky, block.max(8 * 16)))
}
impl ParticleEngine {
    /// Recover the fixed burst origin from its live particles' linear motion.
    /// Deduplicate all 8/24 sprites into one light, with no retained world state.
    pub fn lunatic_orb_origins(&self) -> Vec<([f64; 3], u8)> {
        let mut origins = std::collections::BTreeMap::new();
        for p in self.particles.iter().chain(&self.pending) {
            if p.removed || p.age >= p.lifetime {
                continue;
            }
            let Sprite::Named(name) = &p.sprite else {
                continue;
            };
            let Some(name) = name.strip_prefix(PREFIX) else {
                continue;
            };
            let Some(variant) = ["precision_", "homing_", "bouncing_"]
                .iter()
                .position(|prefix| name.starts_with(prefix))
            else {
                continue;
            };
            let at: [f64; 3] =
                std::array::from_fn(|i| p.position[i] - p.velocity[i] * f64::from(p.age.max(0)));
            let key = (
                variant,
                (at[0] * 4096.0).round() as i64,
                (at[1] * 4096.0).round() as i64,
                (at[2] * 4096.0).round() as i64,
            );
            origins.entry(key).or_insert((at, variant as u8));
        }
        origins.into_values().collect()
    }
    /// One server-approved burst; randomized locally, never one packet per particle.
    pub fn lunatic_orb_burst(
        &mut self,
        at: [f64; 3],
        dir: [f64; 3],
        normal: [i8; 3],
        variant: u8,
        bounce: bool,
    ) {
        let Some(name) = ["precision", "homing", "bouncing"].get(usize::from(variant)) else {
            return;
        };
        let length = dir.iter().map(|v| v * v).sum::<f64>().sqrt();
        let dir = if length > 1e-12 {
            dir.map(|v| v / length)
        } else {
            [0.0, 1.0, 0.0]
        };
        let center =
            std::array::from_fn(|i| at[i] + f64::from(normal[i]) * if bounce { 0.10 } else { 0.5 });
        for sprite in 1..=8 {
            for _ in 0..if bounce { 1 } else { 3 } {
                let mut p = Particle::blank(ParticleType::EndRod, center);
                p.sprite = Sprite::Named(format!("{PREFIX}{name}_{sprite}"));
                // Luanti size units are tenths of a block; Minecraft quadSize is half-width.
                let (min, max) = if bounce { (0.5, 1.0) } else { (1.0, 2.0) };
                p.size = (min + (max - min) * self.random.next_float()) / 20.0;
                p.lifetime = 20;
                p.age = -(self.random.next_float() * if bounce { 1.6 } else { 2.0 }).round() as i32;
                p.no_render = p.age < 0;
                p.has_physics = false;
                for i in 0..3 {
                    let (low, high) = if bounce {
                        if i == 1 { (0.2, 0.45) } else { (-0.1, 0.1) }
                    } else {
                        let (mut low, mut high) = (dir[i] - 0.4, dir[i] + 0.4);
                        if i == 1 {
                            low = low.abs();
                            high = high.abs();
                        }
                        (low.min(high), low.max(high))
                    };
                    p.velocity[i] = (low + (high - low) * self.random.next_double()) / 20.0;
                }
                self.pending.push(p);
                *self
                    .made
                    .entry(if bounce {
                        "lunatic:orb_bounce"
                    } else {
                        "lunatic:orb_impact"
                    })
                    .or_default() += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    #[test]
    fn one_light_stays_at_origin_until_its_last_particle_expires() {
        let mut e = ParticleEngine::new(
            std::path::Path::new("/nonexistent"),
            &HashMap::new(),
            &HashSet::new(),
        );
        e.lunatic_orb_burst([1.5, 70.0, 2.5], [0.0, 0.0, 1.0], [0, 1, 0], 2, false);
        assert_eq!(e.lunatic_orb_origins(), vec![([1.5, 70.5, 2.5], 2)]);
        for p in &mut e.pending {
            while p.age < 0 {
                tick(p);
            }
            for _ in 0..8 {
                tick(p);
            }
        }
        let origins = e.lunatic_orb_origins();
        assert_eq!(origins.len(), 1);
        for (actual, expected) in origins[0].0.into_iter().zip([1.5, 70.5, 2.5]) {
            assert!((actual - expected).abs() < 1e-9);
        }
        for p in &mut e.pending {
            for _ in 0..20 {
                tick(p);
            }
        }
        assert!(e.lunatic_orb_origins().is_empty());
        e.clear();
        assert!(e.lunatic_orb_origins().is_empty());
    }
    #[test]
    fn luanti_burst_counts_sizes_lifetime_and_motion() {
        let mut e = ParticleEngine::new(
            std::path::Path::new("/nonexistent"),
            &HashMap::new(),
            &HashSet::new(),
        );
        e.lunatic_orb_burst([0.0; 3], [0.0, 0.0, 1.0], [0, 1, 0], 1, false);
        assert_eq!(e.pending.len(), 24);
        for p in &e.pending {
            assert_eq!(p.lifetime, 20);
            assert!((0.05..=0.1).contains(&p.size));
            assert!((0.03..=0.07).contains(&p.velocity[2]));
            assert_eq!(p.position, [0.0, 0.5, 0.0]);
        }
        e.pending.clear();
        e.lunatic_orb_burst([0.0; 3], [0.0; 3], [1, 0, 0], 2, true);
        assert_eq!(e.pending.len(), 8);
        for mut p in e.pending {
            assert!((0.025..=0.05).contains(&p.size));
            assert!((0.01..=0.0225).contains(&p.velocity[1]));
            assert_eq!(light(&p, 0, 0), Some((0, 128)));
            let start = p.position;
            while p.age < 0 {
                tick(&mut p);
                assert_eq!(p.position, start);
            }
            for _ in 0..20 {
                tick(&mut p);
            }
            assert!(p.removed);
        }
        let mut vanilla = Particle::blank(ParticleType::EndRod, [0.0; 3]);
        assert!(!tick(&mut vanilla));
        assert_eq!(light(&vanilla, 0, 0), None);
    }
}
