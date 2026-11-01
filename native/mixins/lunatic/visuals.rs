// SPDX-License-Identifier: GPL-3.0-or-later
// Luanti's purification_projectile.pointlight, tinted to each orb's inventory color.
// Read client entities each frame: no packets, saved light writes or retained effects.
use super::{Game, native_hooks::NativeColoredLight};
use horizon_client::projectiles::{Projectile, lunatic_motion};
use horizon_core::geometry::Vec3;

fn lights(
    projectiles: &[Projectile],
    origin: Vec3,
    alpha: f64,
    loaded: impl Fn((i32, i32)) -> bool,
) -> Vec<NativeColoredLight> {
    let mut nearest: Vec<(f64, NativeColoredLight)> = Vec::with_capacity(8);
    for p in projectiles {
        let Some(variant) = lunatic_motion::variant(p) else {
            continue;
        };
        // Match entity rendering, including paused/frozen-world interpolation.
        let at = p.previous.lerp(p.position, alpha);
        let distance =
            (at.x - origin.x).powi(2) + (at.y - origin.y).powi(2) + (at.z - origin.z).powi(2);
        if !distance.is_finite()
            || distance > 32.0 * 32.0
            || !loaded((
                (at.x.floor() as i32).div_euclid(16),
                (at.z.floor() as i32).div_euclid(16),
            ))
        {
            continue;
        }
        let index = nearest.partition_point(|(d, _)| *d <= distance);
        if index >= 8 {
            continue;
        }
        nearest.insert(
            index,
            (
                distance,
                NativeColoredLight {
                    at: [at.x, at.y, at.z],
                    rgb: [0xdc143c, 0x800080, 0x00ff00][variant],
                    radius: 6.0,
                    intensity: 0.5,
                },
            ),
        );
        nearest.truncate(8);
    }
    nearest.into_iter().map(|(_, light)| light).collect()
}

impl Game<'_> {
    pub(super) fn lunatic_visuals(&self, out: &mut Vec<NativeColoredLight>) {
        if !self.native_on("lunatic") {
            return;
        }
        let Some(player) = &self.player else {
            return;
        };
        out.extend(lights(
            &self.creatures.projectiles,
            player.position,
            self.clock.fraction(),
            |chunk| self.world.chunks.contains_key(&chunk),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_client::projectiles::{ProjectileKind, ThrownType};
    static TYPES: [ThrownType; 3] = [
        ThrownType {
            id: lunatic_motion::ITEMS[0],
            item: lunatic_motion::ITEMS[0],
            damage: 1.0,
        },
        ThrownType {
            id: lunatic_motion::ITEMS[1],
            item: lunatic_motion::ITEMS[1],
            damage: 0.5,
        },
        ThrownType {
            id: lunatic_motion::ITEMS[2],
            item: lunatic_motion::ITEMS[2],
            damage: 2.0,
        },
    ];
    fn orb(kind: usize, x: f64) -> Projectile {
        Projectile::new(
            ProjectileKind::Thrown(&TYPES[kind]),
            Vec3::new(x, 0.0, 0.0),
            None,
        )
    }
    #[test]
    fn luanti_pointlight_for_all_three_variants() {
        let orbs = [orb(0, 1.0), orb(1, 2.0), orb(2, 3.0)];
        let out = lights(&orbs, Vec3::ZERO, 1.0, |_| true);
        assert_eq!(out.len(), 3);
        for (l, rgb) in out.iter().zip([0xdc143c, 0x800080, 0x00ff00]) {
            assert_eq!((l.rgb, l.radius, l.intensity), (rgb, 6.0, 0.5));
        }
    }
    #[test]
    fn nearest_loaded_sources_follow_render_interpolation() {
        let mut orbs: Vec<_> = (1..=12).rev().map(|x| orb(0, f64::from(x))).collect();
        for p in &mut orbs {
            p.previous.x = p.position.x - 2.0;
        }
        // Unloaded sources must not consume the eight-request budget.
        orbs.push(orb(1, -1.0));
        orbs.push(orb(2, 100.0));
        let out = lights(&orbs, Vec3::ZERO, 0.5, |(x, _)| x >= 0);
        assert_eq!(
            out.iter().map(|l| l.at[0]).collect::<Vec<_>>(),
            (0..8).map(f64::from).collect::<Vec<_>>()
        );
    }
    #[test]
    fn no_stale_lights_or_vanilla_projectile_lights() {
        let snowball = Projectile::new(ProjectileKind::Snowball, Vec3::ZERO, None);
        assert!(lights(&[snowball], Vec3::ZERO, 1.0, |_| true).is_empty());
        assert_eq!(lights(&[orb(0, 0.0)], Vec3::ZERO, 1.0, |_| true).len(), 1);
        assert!(lights(&[], Vec3::ZERO, 1.0, |_| true).is_empty());
    }
}
